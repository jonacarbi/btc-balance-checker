# Testing

| Suite | Command | Count (2026-10-09) |
|---|---|---|
| Rust unit | `cargo test --manifest-path src-tauri/Cargo.toml` | 29 passed |
| Node validator | `node --test tests/*.test.mjs` | 4 passed |
| Playwright desktop + web + site | `npx playwright test` | 22 passed, 1 skipped |
| Live lookup | `LIVE=1 npx playwright test -g live` | 1 passed |

Desktop UI specs run against a stubbed Tauri API. `tests/sample-addresses.txt` holds public addresses only (C-001).
