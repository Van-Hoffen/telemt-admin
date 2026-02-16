//! Inline-кнопки для approve/reject.

use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;

pub fn approve_reject_buttons(request_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::callback("✅ Одобрить", format!("approve:{}", request_id)),
        InlineKeyboardButton::callback("❌ Отклонить", format!("reject:{}", request_id)),
    ])
}
