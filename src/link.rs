//! Генерация fake-TLS ссылок на прокси telemt.

use crate::telemt_cfg::TelemtLinkParams;
use rand::RngCore;
use std::fmt::Write;

/// Генерирует 32 hex-символа (16 байт) для секрета пользователя.
pub fn generate_user_secret() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Формирует fake-TLS секрет: ee + user_secret (32 hex) + hex(tls_domain).
pub fn build_fake_tls_secret(user_secret: &str, tls_domain: &str) -> String {
    let domain_hex = hex::encode(tls_domain.as_bytes());
    let mut s = String::with_capacity(2 + 32 + domain_hex.len());
    s.push_str("ee");
    s.push_str(user_secret);
    s.push_str(&domain_hex);
    s
}

/// Формирует tg://proxy ссылку.
pub fn build_proxy_link(
    params: &TelemtLinkParams,
    user_secret: &str,
) -> Result<String, std::fmt::Error> {
    let secret = build_fake_tls_secret(user_secret, &params.tls_domain);
    let mut url = String::new();
    write!(
        url,
        "tg://proxy?server={}&port={}&secret={}",
        params.host, params.port, secret
    )?;
    Ok(url)
}
