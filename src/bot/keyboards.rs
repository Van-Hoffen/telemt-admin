//! Клавиатуры бота: inline и постоянные reply-кнопки.

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

pub const BTN_USER_LINK: &str = "🔗 Моя ссылка";
pub const BTN_USER_GUIDE: &str = "❓ Инструкция";

pub const BTN_ADMIN_PENDING: &str = "📥 Новые заявки";
pub const BTN_ADMIN_USERS: &str = "👥 Список пользователей";
pub const BTN_ADMIN_SERVICE: &str = "⚙️ Статус сервиса";
pub const BTN_ADMIN_STATS: &str = "📊 Статистика";
pub const BTN_ADMIN_CREATE_HINT: &str = "➕ Создать @username";
pub const BTN_ADMIN_HELP: &str = "❓ Справка";

pub fn user_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(BTN_USER_LINK),
        KeyboardButton::new(BTN_USER_GUIDE),
    ]])
    .resize_keyboard()
    .persistent()
}

pub fn admin_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(BTN_ADMIN_PENDING),
            KeyboardButton::new(BTN_ADMIN_USERS),
        ],
        vec![
            KeyboardButton::new(BTN_ADMIN_SERVICE),
            KeyboardButton::new(BTN_ADMIN_STATS),
        ],
        vec![
            KeyboardButton::new(BTN_ADMIN_CREATE_HINT),
            KeyboardButton::new(BTN_ADMIN_HELP),
        ],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn approve_reject_buttons(request_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::callback("✅ Одобрить", format!("approve:{}", request_id)),
        InlineKeyboardButton::callback("❌ Отклонить", format!("reject:{}", request_id)),
    ])
}

pub fn users_page_keyboard(
    users: &[(i64, String)],
    page: i64,
    total_pages: i64,
) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for (tg_user_id, title) in users {
        rows.push(vec![InlineKeyboardButton::callback(
            format!("👤 {}", title),
            format!("user_open:{}:{}", tg_user_id, page),
        )]);
    }

    let prev_page = if page > 1 { page - 1 } else { 1 };
    let next_page = if page < total_pages {
        page + 1
    } else {
        total_pages
    };

    rows.push(vec![
        InlineKeyboardButton::callback("⬅️", format!("users_page:{}", prev_page)),
        InlineKeyboardButton::callback(
            format!("📄 {}/{}", page, total_pages.max(1)),
            format!("users_page:{}", page),
        ),
        InlineKeyboardButton::callback("➡️", format!("users_page:{}", next_page)),
    ]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🔄 Обновить",
        format!("users_page:{}", page),
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub fn user_card_keyboard(tg_user_id: i64, page: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default()
        .append_row(vec![InlineKeyboardButton::callback(
            "🔗 Данные + QR",
            format!("user_view:{}:{}", tg_user_id, page),
        )])
        .append_row(vec![InlineKeyboardButton::callback(
            "⛔ Забанить (удалить)",
            format!("user_ban:{}:{}", tg_user_id, page),
        )])
        .append_row(vec![InlineKeyboardButton::callback(
            "⬅️ Назад к списку",
            format!("users_page:{}", page),
        )])
}

pub fn service_control_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default()
        .append_row(vec![
            InlineKeyboardButton::callback("🔄 Обновить", "service:status"),
            InlineKeyboardButton::callback("♻️ Рестарт", "service:restart"),
        ])
}
