# Project overview

Stack: Rust 2021 + Tauri 2, plain HTML/CSS/JS frontend (no framework), Playwright tests, Netlify static hosting, GitHub Actions builds.

| Path | Contents |
|---|---|
| `src-tauri/src/address.rs` | Base58Check and bech32/bech32m validation, bulk parsing |
| `src-tauri/src/api.rs` | blockchain.info batch, mempool.space fallback, 429 backoff |
| `src-tauri/src/scan.rs` | One scan pass: fetch, persist, alert |
| `src-tauri/src/store.rs` | Atomic persistence, seen set, log |
| `src-tauri/src/alerts.rs` | Telegram and SMTP |
| `src-tauri/src/settings.rs` | Settings file, legacy `.env` import |
| `src-tauri/src/gui.rs` / `cli.rs` / `main.rs` | Tauri commands and watch loop / `--cli` loop / entry |
| `src/` | Desktop frontend; owns `ledger.css`, fonts, logo |
| `site/` | Landing page; `site/app/` web app |
| `tests/` | Node and Playwright tests |
| `legacy/` | Original Python script |
