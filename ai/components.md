# Components

| Component | Where | Contract |
|---|---|---|
| Add single | `src/index.html`, `site/app/index.html` | Validates on submit, reports invalid inline |
| Bulk paste | same | Splits on whitespace, comma, semicolon; reports added, duplicates, invalid |
| Drop zone | same | Focusable, Enter opens file picker, accepts .txt/.csv only |
| Ledger table | same | Row number, address, BTC, sats, status, last checked, remove |
| Total figure | same | Sum in BTC, mono figures |
| Watch toggle + interval | desktop only | Non aborting: stop takes effect after the running scan |
| Settings panel | desktop only | Telegram, SMTP, test alert, import legacy `.env` |
