# Product

Watches a list of Bitcoin addresses and alerts once, by Telegram and email, the first time an address shows a balance. Audience: the owner and anyone tracking many addresses without exposing keys.

| Surface | Features |
|---|---|
| Desktop app | Add one, bulk paste, drag and drop .txt/.csv, checksum validation, dedupe, balances in BTC and sats, total, export, check now, watch mode (default 60 s), alerts, settings, `.env` import |
| CLI | `btc-balance-checker --cli file [--once]` |
| Web app | One off checks via mempool.space, no alerts |
