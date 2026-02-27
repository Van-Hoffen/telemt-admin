//! Клавиатуры бота: inline и постоянные reply-кнопки.

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

pub const BTN_USER_LINK: &str = "🔗 Моя ссылка";
pub const BTN_USER_GUIDE: &str = "❓ Инструкция";

pub const BTN_ADMIN_REQUESTS: &str = "📋 Управление заявками";
pub const BTN_ADMIN_TOKENS: &str = "🔑 Управление токенами";
pub const BTN_ADMIN_USERS: &str = "👥 Список пользователей";
pub const BTN_ADMIN_SERVICE: &str = "⚙️ Статус сервиса";
pub const BTN_ADMIN_STATS: &str = "📊 Статистика";
pub const BTN_ADMIN_CREATE_HINT: &str = "➕ Создать @username";
pub const BTN_ADMIN_HELP: &str = "❓ Справка";

// Подменю для управления заявками
pub const BTN_ADMIN_PENDING: &str = "📥 Новые заявки";

// Подменю для управления токенами
pub const BTN_ADMIN_TOKEN_CREATE: &str = "➕ Создать токен";
pub const BTN_ADMIN_TOKEN_LIST: &str = "📋 Список токенов";

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
            KeyboardButton::new(BTN_ADMIN_REQUESTS),
            KeyboardButton::new(BTN_ADMIN_TOKENS),
        ],
        vec![
            KeyboardButton::new(BTN_ADMIN_USERS),
            KeyboardButton::new(BTN_ADMIN_SERVICE),
        ],
        vec![
            KeyboardButton::new(BTN_ADMIN_STATS),
            KeyboardButton::new(BTN_ADMIN_CREATE_HINT),
        ],
        vec![
            KeyboardButton::new(BTN_ADMIN_HELP),
        ],
    ])
    .resize_keyboard()
    .persistent()
}

// Подменю для управления заявками
pub fn admin_requests_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(BTN_ADMIN_PENDING),
        ],
        vec![
            KeyboardButton::new("◀️ Назад"),
        ],
    ])
    .resize_keyboard()
    .persistent()
}

// Подменю для управления токенами
pub fn admin_tokens_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(BTN_ADMIN_TOKEN_CREATE),
            KeyboardButton::new(BTN_ADMIN_TOKEN_LIST),
        ],
        vec![
            KeyboardButton::new("◀️ Назад"),
        ],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn token_list_keyboard(
    tokens: &[String], // Список токенов
    page: i64,
    total_pages: i64,
) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    
    // Добавляем токены с возможностью отозвать
    for token in tokens {
        rows.push(vec![
            InlineKeyboardButton::callback(
                format!("🔑 {}", token),
                format!("token:view:{}", token), // Просмотр информации о токене
            ),
            InlineKeyboardButton::callback(
                "🚫 Отозвать".to_string(),
                format!("token:revoke:{}", token), // Отзыв токена
            ),
        ]);
    }

    // Навигация по страницам
    let prev_page = if page > 1 { page - 1 } else { 1 };
    let next_page = if page < total_pages {
        page + 1
    } else {
        total_pages
    };

    rows.push(vec![
        InlineKeyboardButton::callback(
            "⬅️".to_string(),
            format!("tokens_page:{}", prev_page),
        ),
        InlineKeyboardButton::callback(
            format!("📄 {}/{}", page, total_pages.max(1)),
            format!("tokens_page:{}", page),
        ),
        InlineKeyboardButton::callback(
            "➡️".to_string(),
            format!("tokens_page:{}", next_page),
        ),
    ]);
    
    // Кнопка обновления списка
    rows.push(vec![InlineKeyboardButton::callback(
        "🔄 Обновить".to_string(),
        format!("tokens_page:{}", page),
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub fn service_control_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default()
        .append_row(vec![
            InlineKeyboardButton::callback("🔄 Обновить", "service:status"),
            InlineKeyboardButton::callback("♻️ Рестарт", "service:restart"),
        ])
}

pub fn approve_reject_buttons(request_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::callback("✅ Одобрить", format!("approve:{}", request_id)),
        InlineKeyboardButton::callback("❌ Отклонить", format!("reject:{}", request_id)),
    ])
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
