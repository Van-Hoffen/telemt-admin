//! Обработчики команд пользователя и админа.

use crate::config::Config;
use crate::db::{Db, RegisterResult, RegistrationRequest};
use crate::link::{build_proxy_link, generate_user_secret};
use crate::service::ServiceController;
use crate::telemt_cfg::TelemtConfig;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct BotState {
    pub config: Arc<Config>,
    pub db: Arc<Db>,
    pub telemt_cfg: Arc<TelemtConfig>,
    pub service: ServiceController,
}

fn telemt_username(tg_user_id: i64) -> String {
    format!("tg_{}", tg_user_id)
}

fn sender_user_id(msg: &Message) -> Option<i64> {
    msg.from.as_ref().map(|user| user.id.0 as i64)
}

fn is_admin_message(msg: &Message, state: &BotState) -> bool {
    sender_user_id(msg).is_some_and(|user_id| state.config.is_admin(user_id))
}

fn parse_callback_request_id(data: &str, prefix: &str) -> Result<i64, anyhow::Error> {
    data.strip_prefix(prefix)
        .ok_or_else(|| anyhow::anyhow!("Некорректный callback payload"))?
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Некорректный request_id"))
}

fn callback_message_target(q: &CallbackQuery) -> Option<(ChatId, teloxide::types::MessageId)> {
    q.message.as_ref().map(|msg| (msg.chat().id, msg.id()))
}

async fn approve_request_and_build_link(
    state: &BotState,
    request_id: i64,
) -> Result<Option<(RegistrationRequest, String)>, anyhow::Error> {
    let request = match state.db.get_pending_by_id(request_id).await? {
        Some(request) => request,
        None => return Ok(None),
    };

    let telemt_user = telemt_username(request.tg_user_id);
    let user_secret = generate_user_secret();

    state.telemt_cfg.upsert_user(&telemt_user, &user_secret)?;
    if state
        .db
        .approve(request_id, &telemt_user, &user_secret)
        .await?
        .is_none()
    {
        return Ok(None);
    }

    let link_params = state.telemt_cfg.read_link_params()?;
    let proxy_link = build_proxy_link(&link_params, &user_secret)?;
    Ok(Some((request, proxy_link)))
}

async fn start_cmd(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    let user_id = sender_user_id(&msg).unwrap_or_default();
    let username = msg.from.as_ref().and_then(|u| u.username.clone());
    tracing::info!(
        user_id = user_id,
        username = ?username,
        "Received /start command"
    );

    let result = state
        .db
        .register_or_get(user_id, username.as_deref())
        .await?;

    match result {
        RegisterResult::Approved(secret) => {
            let params = state.telemt_cfg.read_link_params()?;
            let link = build_proxy_link(&params, &secret)?;
            bot.send_message(msg.chat.id, format!("Ваша ссылка на прокси:\n\n{}", link))
                .await?;
            return Ok(());
        }
        RegisterResult::Rejected => {
            bot.send_message(
                msg.chat.id,
                "Ваша заявка на регистрацию отклонена администратором.",
            )
            .await?;
            return Ok(());
        }
        RegisterResult::AlreadyPending => {
            bot.send_message(
                msg.chat.id,
                "Ваша заявка уже на рассмотрении. Ожидайте подтверждения администратора.",
            )
            .await?;
            return Ok(());
        }
        RegisterResult::NewPending(ref req) => {
            bot.send_message(
                msg.chat.id,
                "Заявка на регистрацию отправлена администратору. Ожидайте подтверждения.",
            )
            .await?;
            notify_admins(&bot, &state, req).await?;
        }
    }
    Ok(())
}

async fn notify_admins(bot: &Bot, state: &BotState, req: &RegistrationRequest) -> HandlerResult {
    let text = format!(
        "📋 Новая заявка #{}:\n\
         User ID: {}\n\
         Username: @{}\n\
         Время: {}",
        req.id,
        req.tg_user_id,
        req.tg_username.as_deref().unwrap_or("—"),
        format_timestamp(req.created_at),
    );

    let kb = crate::bot::keyboards::approve_reject_buttons(req.id);

    for admin_id in &state.config.admin_ids {
        if let Err(e) = bot
            .send_message(ChatId(*admin_id), text.clone())
            .reply_markup(kb.clone())
            .await
        {
            tracing::warn!(
                "Не удалось отправить уведомление админу {}: {}",
                admin_id,
                e
            );
        }
    }
    Ok(())
}

fn format_timestamp(ts: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let d = UNIX_EPOCH + Duration::from_secs(ts as u64);
    format!("{:?}", d)
}

async fn callback_approve(bot: Bot, q: CallbackQuery, state: BotState) -> HandlerResult {
    let callback_id = q.id.clone();
    let admin_id = q.from.id.0 as i64;
    if !state.config.is_admin(admin_id) {
        bot.answer_callback_query(callback_id)
            .text("Недостаточно прав")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let data = q.data.as_deref().unwrap_or("");
    let request_id = parse_callback_request_id(data, "approve:")?;
    tracing::info!(
        admin_id = admin_id,
        request_id = request_id,
        "Approve callback received"
    );
    let message_target = callback_message_target(&q);

    let (request, link) = match approve_request_and_build_link(&state, request_id).await? {
        Some(payload) => payload,
        None => {
            bot.answer_callback_query(callback_id)
                .text("Заявка уже обработана или не найдена")
                .await?;
            return Ok(());
        }
    };

    bot.answer_callback_query(q.id).text("Одобрено").await?;

    if let Some((chat_id, message_id)) = message_target {
        bot.edit_message_text(chat_id, message_id, "✅ Заявка одобрена")
            .reply_markup(teloxide::types::InlineKeyboardMarkup::default())
            .await?;
    }

    bot.send_message(
        ChatId(request.tg_user_id),
        format!("Ваша ссылка на прокси:\n\n{}", link),
    )
    .await?;

    tracing::info!("Admin {} approved request #{}", admin_id, request_id);
    Ok(())
}

async fn callback_reject(bot: Bot, q: CallbackQuery, state: BotState) -> HandlerResult {
    let callback_id = q.id.clone();
    let admin_id = q.from.id.0 as i64;
    if !state.config.is_admin(admin_id) {
        bot.answer_callback_query(callback_id)
            .text("Недостаточно прав")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let data = q.data.as_deref().unwrap_or("");
    let request_id = parse_callback_request_id(data, "reject:")?;
    tracing::info!(
        admin_id = admin_id,
        request_id = request_id,
        "Reject callback received"
    );
    let message_target = callback_message_target(&q);
    let request = state.db.reject(request_id).await?;

    bot.answer_callback_query(q.id).text("Отклонено").await?;

    if let Some(request) = request {
        if let Some((chat_id, message_id)) = message_target {
            bot.edit_message_text(chat_id, message_id, "❌ Заявка отклонена")
                .reply_markup(teloxide::types::InlineKeyboardMarkup::default())
                .await?;
        }
        bot.send_message(
            ChatId(request.tg_user_id),
            "Ваша заявка на регистрацию отклонена администратором.",
        )
        .await?;
    }

    tracing::info!("Admin {} rejected request #{}", admin_id, request_id);
    Ok(())
}

async fn cmd_approve(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    if !is_admin_message(&msg, &state) {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let request_id: i64 = match text.split_whitespace().nth(1).unwrap_or("").parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "Использование: /approve <request_id>")
                .await?;
            return Ok(());
        }
    };
    tracing::info!(request_id = request_id, "Admin command /approve");

    let (request, link) = match approve_request_and_build_link(&state, request_id).await? {
        Some(payload) => payload,
        None => {
            bot.send_message(msg.chat.id, "Заявка не найдена или уже обработана")
                .await?;
            return Ok(());
        }
    };

    bot.send_message(
        msg.chat.id,
        format!("Одобрено. Ссылка отправлена пользователю.\n{}", link),
    )
    .await?;
    bot.send_message(
        ChatId(request.tg_user_id),
        format!("Ваша ссылка на прокси:\n\n{}", link),
    )
    .await?;
    Ok(())
}

async fn cmd_reject(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    if !is_admin_message(&msg, &state) {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let request_id: i64 = match text.split_whitespace().nth(1).unwrap_or("").parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "Использование: /reject <request_id>")
                .await?;
            return Ok(());
        }
    };
    tracing::info!(request_id = request_id, "Admin command /reject");

    let req = state.db.reject(request_id).await?;
    if let Some(r) = req {
        bot.send_message(msg.chat.id, "Заявка отклонена").await?;
        bot.send_message(
            ChatId(r.tg_user_id),
            "Ваша заявка на регистрацию отклонена администратором.",
        )
        .await?;
    } else {
        bot.send_message(msg.chat.id, "Заявка не найдена или уже обработана")
            .await?;
    }
    Ok(())
}

async fn cmd_create(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    if !is_admin_message(&msg, &state) {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let tg_user_id: i64 = match text.split_whitespace().nth(1).unwrap_or("").parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "Использование: /create <telegram_user_id>")
                .await?;
            return Ok(());
        }
    };
    tracing::info!(tg_user_id = tg_user_id, "Admin command /create");

    let telemt_user = telemt_username(tg_user_id);
    let secret = generate_user_secret();

    state.telemt_cfg.upsert_user(&telemt_user, &secret)?;
    state
        .db
        .set_approved(tg_user_id, &telemt_user, &secret)
        .await?;

    let params = state.telemt_cfg.read_link_params()?;
    let link = build_proxy_link(&params, &secret)?;

    bot.send_message(
        msg.chat.id,
        format!("Пользователь {} создан.\nСсылка:\n{}", telemt_user, link),
    )
    .await?;
    Ok(())
}

async fn cmd_delete(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    if !is_admin_message(&msg, &state) {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let tg_user_id: i64 = match text.split_whitespace().nth(1).unwrap_or("").parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "Использование: /delete <telegram_user_id>")
                .await?;
            return Ok(());
        }
    };
    tracing::info!(tg_user_id = tg_user_id, "Admin command /delete");

    let telemt_user = telemt_username(tg_user_id);
    let removed = state.telemt_cfg.remove_user(&telemt_user)?;
    let _ = state.db.deactivate_user(tg_user_id).await;

    if removed {
        bot.send_message(msg.chat.id, format!("Пользователь {} удалён", telemt_user))
            .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            format!("Пользователь {} не найден в конфиге", telemt_user),
        )
        .await?;
    }
    Ok(())
}

async fn cmd_service(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    if !is_admin_message(&msg, &state) {
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args: Vec<&str> = text.split_whitespace().collect();
    let action = args.get(1).copied().unwrap_or("status");
    tracing::info!(action = action, "Admin command /service");

    let (action_name, result) = match action {
        "start" => ("start", state.service.start()),
        "stop" => ("stop", state.service.stop()),
        "restart" => ("restart", state.service.restart()),
        "status" => ("status", state.service.status()),
        _ => {
            bot.send_message(
                msg.chat.id,
                "Использование: /service <start|stop|restart|status>",
            )
            .await?;
            return Ok(());
        }
    };

    let reply = state.service.format_result(action_name, &result);
    bot.send_message(msg.chat.id, reply).await?;
    Ok(())
}

async fn cmd_link(bot: Bot, msg: Message, state: BotState) -> HandlerResult {
    let user_id = sender_user_id(&msg).unwrap_or_default();
    tracing::info!(user_id = user_id, "Received /link command");

    let maybe = state.db.get_approved(user_id).await?;
    match maybe {
        Some((_, secret)) => {
            let params = state.telemt_cfg.read_link_params()?;
            let link = build_proxy_link(&params, &secret)?;
            bot.send_message(msg.chat.id, format!("Ваша ссылка на прокси:\n\n{}", link))
                .await?;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "У вас нет доступа к прокси. Отправьте /start для регистрации.",
            )
            .await?;
        }
    }
    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum BotCommand {
    #[command(description = "Зарегистрироваться")]
    Start,
    #[command(description = "Получить ссылку на прокси")]
    Link,
    #[command(description = "Справка")]
    Help,
    #[command(description = "Одобрить заявку (админ)")]
    Approve,
    #[command(description = "Отклонить заявку (админ)")]
    Reject,
    #[command(description = "Создать пользователя (админ)")]
    Create,
    #[command(description = "Удалить пользователя (админ)")]
    Delete,
    #[command(description = "Управление сервисом (админ)")]
    Service,
}

async fn cmd_help(bot: Bot, msg: Message) -> HandlerResult {
    let text = r#"Команды:
/start — зарегистрироваться (заявка на подтверждение админу)
/link — получить ссылку на прокси (если уже одобрены)

Для администраторов:
/approve <id> — одобрить заявку
/reject <id> — отклонить заявку
/create <tg_user_id> — создать пользователя
/delete <tg_user_id> — удалить пользователя
/service <start|stop|restart|status> — управление telemt.service"#;
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

pub fn schema() -> dptree::Handler<
    'static,
    Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
    DpHandlerDescription,
> {
    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(dptree::case![BotCommand::Start].endpoint(start_cmd))
        .branch(dptree::case![BotCommand::Link].endpoint(cmd_link))
        .branch(dptree::case![BotCommand::Help].endpoint(cmd_help))
        .branch(dptree::case![BotCommand::Approve].endpoint(cmd_approve))
        .branch(dptree::case![BotCommand::Reject].endpoint(cmd_reject))
        .branch(dptree::case![BotCommand::Create].endpoint(cmd_create))
        .branch(dptree::case![BotCommand::Delete].endpoint(cmd_delete))
        .branch(dptree::case![BotCommand::Service].endpoint(cmd_service));

    let callback_handler = Update::filter_callback_query()
        .branch(
            dptree::filter_map(|q: CallbackQuery| {
                if q.data
                    .as_deref()
                    .is_some_and(|payload| payload.starts_with("approve:"))
                {
                    Some(q)
                } else {
                    None
                }
            })
            .endpoint(callback_approve),
        )
        .branch(
            dptree::filter_map(|q: CallbackQuery| {
                if q.data
                    .as_deref()
                    .is_some_and(|payload| payload.starts_with("reject:"))
                {
                    Some(q)
                } else {
                    None
                }
            })
            .endpoint(callback_reject),
        );

    let message_handler = Update::filter_message().branch(command_handler);

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}
