# BTC Balance Checker - Web checker

One-off Bitcoin balance check that runs entirely in your browser (static page, no backend).

- Input: a single address, a pasted list (newline, comma or space separated) or a dropped `.txt` / `.csv` file.
- Validation: real checksum verification for P2PKH, P2SH, bech32 and bech32m addresses; invalid ones are listed, duplicates removed. Limit 500 addresses.
- Lookup: each address is fetched from `https://mempool.space/api/address/<address>`; the balance is confirmed plus unconfirmed, shown in BTC and sats with a total.
- Not included: watch mode, Telegram/email alerts, saved lists, secrets. For those, download the desktop app: https://github.com/jonacarbi/btc-balance-checker/releases
