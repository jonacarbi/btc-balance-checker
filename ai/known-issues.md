# Known issues

| ID | Severity | Status | Summary |
|---|---|---|---|
| KI-001 | Medium | Open | Builds are unsigned and not notarized |
| KI-002 | Low | Open | Netlify injects a HUD script that the CSP blocks (console noise only) |
| KI-003 | Low | Open | `balance` command still runs the legacy Python script |
| KI-004 | Low | Open | Windows and Linux bundles and `AttachConsole` not run locally |
| KI-005 | Low | Open | Watch mode state resets on app restart |
| KI-006 | Low | Open | Telegram and SMTP delivery not tested against real services |

## KI-001
Observed: no signing config in `tauri.conf.json`. Impact: macOS Gatekeeper and Windows SmartScreen warn. Action: owner supplies certificates.

## KI-002
Observed: 2026-10-09 production console shows 2 CSP errors from `/.netlify/scripts/hud`. Impact: none on function. Action: disable the Netlify drawer in site settings, or ignore.

## KI-003
Observed: `~/.local/bin/balance` execs the venv Python. Action: after install, exec `btc-balance-checker --cli <file>`.

## KI-004 to KI-006
Observed in REPORT of entry A. Action: verify on first CI release and first real alert.

## Resolved
None yet.
