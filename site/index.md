# BTC Balance Checker

Free desktop app that watches Bitcoin addresses and alerts you (Telegram and/or email) the first time an address shows a balance. Windows, macOS, Linux. Open source. Read-only: it handles public addresses only, never private keys.

## Features

- Add addresses one by one, paste in bulk (newline, comma or space separated), or drag and drop `.txt` / `.csv` files.
- Real checksum validation for P2PKH (`1...`), P2SH (`3...`), bech32 (`bc1q...`) and bech32m/Taproot (`bc1p...`). Invalid entries are listed, duplicates removed.
- Per-address balance in BTC and sats, status, last-checked time, and a total.
- "Check now" plus watch mode with a configurable interval (default 60 s). Batched lookups via blockchain.info with mempool.space fallback.
- One alert per address, on first-seen balance: Telegram bot message and SMTP email (Gmail by default). Credentials are stored in the OS config folder with owner-only permissions. Legacy `.env` keys can be imported.
- Export the list as `.txt` or `.csv`. Everything is stored locally.
- CLI: `btc-balance-checker --cli [address-file]` runs the classic terminal watch loop. No arguments starts the GUI.

## Download

Latest installers (Windows `.exe`/`.msi`, macOS universal `.dmg`, Linux `.AppImage`/`.deb`/`.rpm`): https://github.com/jonacarbi/btc-balance-checker/releases

## Web checker

A simplified in-browser version for one-off checks (single, bulk, drag and drop) is at /app/. It queries mempool.space directly from your browser and has no alerts, watch mode or stored data.
