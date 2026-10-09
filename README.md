# BTC Balance Checker v2

Watch Bitcoin addresses in bulk and get a Telegram and/or email alert the first time an address shows a balance. Rust + Tauri 2 desktop app (Windows, macOS, Linux), a lite in-browser checker, and a landing page. Read-only: it only ever handles public addresses.

| Part | Where |
|---|---|
| Desktop app | `src-tauri/` (Rust) + `src/` (plain HTML/CSS/JS) |
| Web checker | `site/app/` (static, mempool.space from the browser) |
| Landing page | `site/index.html`, `site/llms.txt`, `site/*.md` |
| Legacy script | `legacy/JCBalance.py` (kept for reference) |

## Install

Download the installer for your OS from [Releases](https://github.com/jonacarbi/btc-balance-checker/releases): `.dmg` (macOS universal), `.exe` / `.msi` (Windows), `.AppImage` / `.deb` / `.rpm` (Linux).
The builds are unsigned: on macOS right-click the app and choose Open the first time; on Windows choose More info, then Run anyway.

## Use

1. Add addresses: type one, paste many (newline, comma or space separated), or drop a `.txt` / `.csv` (your old `address.txt` works). Addresses are checksum-verified (P2PKH `1...`, P2SH `3...`, bech32 `bc1q...`, Taproot `bc1p...`); invalid ones are listed, duplicates removed.
2. **Check now** for a single pass, or **Watch** to rescan every N seconds (Settings, default 60, minimum 10).
3. Settings: Telegram bot token and chat ID, SMTP login (Gmail app password by default). Use **Import legacy .env** to pull `BTC_ALERT_EMAIL`, `BTC_ALERT_PASSWORD`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` from your old `.env`.
4. Export the list as `.txt` or `.csv` (you choose the file in a save dialog).

Alerts fire once per address, on the first scan that sees a positive balance. Lookups use blockchain.info in batches of 50 and fall back to mempool.space per address on error.

Files (per OS data/config dir, folder `btc-balance-checker`): `addresses.json`, `seen_balances.txt`, `balance_log.txt` (data dir; same formats as the legacy script) and `settings.json` (config dir, mode 0600 on unix).

## CLI

```sh
btc-balance-checker --cli address.txt     # legacy-style terminal loop over that file only (the app's list is untouched)
btc-balance-checker --cli address.txt --once   # one scan, then exit
btc-balance-checker --cli                 # loop over the list saved by the app
```

Alert credentials come from the app's saved settings, then the `BTC_ALERT_*` / `TELEGRAM_*` env vars, then a `.env` next to the address file. To keep a `balance` shell command working, point it at the installed binary (for example on macOS `/Applications/BTC Balance Checker.app/Contents/MacOS/btc-balance-checker --cli "$HOME/address.txt"`). On Windows the CLI attaches to the launching terminal.

Only one copy of the app runs at a time; starting it again focuses the open window. Use Settings, Send test alert to check Telegram/SMTP.

## Build

Requires Rust, Node 20+ and the [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/).

```sh
npm ci
npx tauri dev                    # run
npx tauri build --bundles app    # macOS .app only
npx tauri build                  # all bundles for this OS
npm test                         # cargo test + node unit tests + Playwright
LIVE=1 npx playwright test       # also hit the real mempool.space
```

The design system (`src/ledger.css`, fonts, logo) is owned by `src/`; run `sh scripts/sync-ui.sh` to copy it into `site/` (CI fails if they drift). To regenerate icons: `npx tauri icon site/logo.svg`.

## Release

Push to `main` to build and test. Tag `v*` to attach the installers to a GitHub Release (`.github/workflows/build.yml`). Deploy `site/` with Netlify (`netlify.toml`, publish dir `site`).
