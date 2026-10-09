# Project state

Updated 2026-10-09 (D-004).

**Phase:** v2.0.0 built, site live at https://btc-balance-checker.netlify.app; release binaries come from CI on tag `v2.0.0`.

## Done

| Item | Evidence |
|---|---|
| Rust backend: validation, batch lookup, fallback, 429 backoff, alerts, persistence | OBSERVED: `cargo test` 29 passed (2026-10-09) |
| Desktop UI: single, bulk, drag and drop, export dialog, test alert, watch mode | OBSERVED: Playwright desktop specs (stubbed Tauri API) |
| Web app `/app/` | OBSERVED: live lookup of the genesis address returned 57.58294323 BTC on https://btc-balance-checker.netlify.app/app/ |
| Landing page, `llms.txt`, Markdown twins, logo, favicon | OBSERVED: HTTP 200 for `/`, `/app/`, `/llms.txt`, `/index.md`, `/favicon.svg` |
| macOS `.app` | OBSERVED: `npx tauri build --bundles app` built a 4.22 MiB bundle |
| CLI | OBSERVED: `--cli tests/sample-addresses.txt --once` scanned and exited |
| JS tests | OBSERVED: `node --test` 4 passed, Playwright 22 passed, 1 skipped (live gated) |

## Next

1. Confirm the CI run for tag `v2.0.0` attaches dmg, exe, msi, deb, rpm and AppImage (`deployment.md`).
2. Point `~/.local/bin/balance` at the installed app's `--cli` once the owner installs v2 (KI-003).
3. Code signing and notarization (KI-001).

## Blockers

| Blocker | Owner | Ref |
|---|---|---|
| No signing certificates | Owner | KI-001 |
