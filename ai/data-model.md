# Data model

Stored per OS under folder `btc-balance-checker` (via `dirs`).

| File | Dir | Format |
|---|---|---|
| `addresses.json` | data | JSON list of addresses |
| `seen_balances.txt` | data | One address per line (legacy format) |
| `balance_log.txt` | data | `YYYY-MM-DD HH:MM:SS - address - N sats` (legacy format) |
| `settings.json` | config | Telegram token and chat id, SMTP host, user, password, recipient, interval; mode 0600 |

Writes are atomic: unique temp file opened `create_new` at 0600, then rename.
API: blockchain.info returns `{addr: {final_balance}}`; mempool.space returns `chain_stats` and `mempool_stats` funded minus spent sums.
