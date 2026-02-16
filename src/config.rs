//! Конфигурация telemt-admin бота.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Токен Telegram бота (или через TELOXIDE_TOKEN)
    pub bot_token: Option<String>,
    /// Список Telegram user_id администраторов
    pub admin_ids: Vec<i64>,
    /// Путь к конфигу telemt (по умолчанию /etc/telemt.toml)
    #[serde(default = "default_telemt_config_path")]
    pub telemt_config_path: PathBuf,
    /// Путь к SQLite БД (по умолчанию /var/lib/telemt-admin/state.db)
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
    /// Имя systemd-сервиса telemt
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

fn default_telemt_config_path() -> PathBuf {
    PathBuf::from("/etc/telemt.toml")
}

fn default_db_path() -> PathBuf {
    PathBuf::from("/var/lib/telemt-admin/state.db")
}

fn default_service_name() -> String {
    "telemt.service".to_string()
}

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        tracing::debug!("Loading config from {}", path.display());
        let content = std::fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!("Не удалось прочитать конфиг {}: {}", path.display(), e)
        })?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Ошибка парсинга конфига: {}", e))?;
        tracing::info!(
            admin_count = config.admin_ids.len(),
            telemt_config_path = %config.telemt_config_path.display(),
            db_path = %config.db_path.display(),
            service_name = %config.service_name,
            "Config parsed successfully"
        );
        Ok(config)
    }

    pub fn bot_token(&self) -> Result<String, anyhow::Error> {
        self.bot_token
            .clone()
            .or_else(|| std::env::var("TELOXIDE_TOKEN").ok())
            .ok_or_else(|| {
                anyhow::anyhow!("Не задан bot_token в конфиге и TELOXIDE_TOKEN в окружении")
            })
    }

    pub fn is_admin(&self, user_id: i64) -> bool {
        self.admin_ids.contains(&user_id)
    }
}
