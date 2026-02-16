# telemt-admin

Telegram-бот на Rust для администрирования `telemt`:

- регистрация пользователей через заявку;
- approve/reject админом;
- выдача fake-TLS ссылки;
- управление пользователями в `/etc/telemt.toml`;
- управление `telemt.service` через `systemctl`.

## Требования

- Rust stable (edition 2024)
- Linux-хост с `systemd` для команд `/service`
- Telegram bot token

## Сборка

```bash
cargo build --release
```

## Конфиг

По умолчанию используется `/etc/telemt-admin.toml`.

Пример:

```toml
bot_token = "123456:telegram-token"
admin_ids = [123456789]
telemt_config_path = "/etc/telemt.toml"
db_path = "/var/lib/telemt-admin/state.db"
service_name = "telemt.service"
```

Можно не хранить `bot_token` в файле и передавать его через переменную `TELOXIDE_TOKEN`.

## Локальный запуск

```bash
RUST_LOG=info ./target/release/telemt-admin /etc/telemt-admin.toml
```

## Регистрация как systemd-сервис

1. Соберите бинарь и положите его в `/usr/local/bin/telemt-admin`.
2. Создайте пользователя сервиса:

```bash
sudo useradd --system --home /var/lib/telemt-admin --shell /usr/sbin/nologin telemt-admin
sudo mkdir -p /var/lib/telemt-admin
sudo chown -R telemt-admin:telemt-admin /var/lib/telemt-admin
```

3. Создайте unit-файл `/etc/systemd/system/telemt-admin.service`:

```ini
[Unit]
Description=telemt-admin Telegram bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=telemt-admin
Group=telemt-admin
WorkingDirectory=/var/lib/telemt-admin
ExecStart=/usr/local/bin/telemt-admin /etc/telemt-admin.toml
Restart=always
RestartSec=3
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

4. Включите и запустите сервис:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now telemt-admin.service
sudo systemctl status telemt-admin.service
```

## CI/CD

В репозитории настроены GitHub Actions:

- `CI`: `cargo check` + `cargo clippy` для push/PR;
- `Release`: при теге формата `vX.Y.Z`:
  - сборка артефактов под Linux и Windows;
  - публикация GitHub Release;
  - генерация описания изменений из conventional commits с группировкой.
