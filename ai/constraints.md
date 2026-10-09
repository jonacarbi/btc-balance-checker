# Constraints

- C-001: Never commit real addresses, `.env` values, `address.txt`, `seen_balances.txt` or `balance_log.txt`. `.gitignore` covers them.
- C-002: No hardcoded secrets. Alert credentials live only in the OS config dir `settings.json`, mode 0600.
- C-003: Fonts are Schibsted Grotesk + Spline Sans Mono, self hosted. Never reuse a pair from the owner's other projects.
- C-004: `src/` owns `ledger.css`, fonts and `logo.svg`; `site/` gets copies via `scripts/sync-ui.sh`.
- C-005: CSP stays `'self'` only for script and style; `connect-src` lists only `api.github.com` and `mempool.space`.
- C-006: Authored files under 400 lines, functions under 50 lines.
- C-007: The app is read only: it handles public addresses, never keys or seeds.
