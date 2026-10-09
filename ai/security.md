# Security

- CSP (`netlify.toml`): `default-src 'none'; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self' data:; connect-src https://api.github.com https://mempool.space; frame-ancestors 'none'`.
- Headers: HSTS preload, nosniff, `X-Frame-Options: DENY`, strict referrer, empty permissions policy.
- Secrets: env var names only in docs (`BTC_ALERT_EMAIL`, `BTC_ALERT_PASSWORD`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`, `BTC_ALERT_TO`). Desktop stores them in `settings.json` 0600.
- Temp files never reused, symlinks refused (Codex finding, fixed 2026-10-09).
