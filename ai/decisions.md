# Decisions

## D-001: Rewrite in Rust with Tauri 2 (implemented, 2026-10-09)
Owner: "if it's smart to write the app in rust do that for me". Tauri gives native installers on all 3 OSes at about 4 MiB and lets the web app share the UI CSS. Rejected: Python GUI plus PyInstaller (large binaries, no shared UI).

## D-002: Adversarial build round, Codex judge (implemented, 2026-10-09)
Owner asked for "3 adversarial sonnet 5.5 subagents" with "codex as a judge". Entry A (Swiss ledger) won 45/60 for core correctness; C (editorial) 44, B (instrument panel) 43.

## D-003: Public repo jonacarbi/btc-balance-checker (implemented, 2026-10-09)
Public so landing page download links resolve for visitors. Private data stays out (C-001).

## D-004: Web app uses mempool.space only (implemented, 2026-10-09)
mempool.space sends CORS headers; the web app has no alerts, no watch mode and no secrets. The desktop app batches on blockchain.info and falls back to mempool.space per address.
