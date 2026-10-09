# Orchestrator: working in the BTC Balance Checker repo

Start here, then read `manifest.md`, then `project-state.md`. After that, open only the files your task routes to.

## Ground truth (2026-10-09)

- v2.0.0 is a Rust + Tauri 2 desktop app (Windows, macOS, Linux) plus a static Netlify site with a landing page and a simplified web app. Repo: https://github.com/jonacarbi/btc-balance-checker (public, `main`). Site: https://btc-balance-checker.netlify.app.
- It replaces `legacy/JCBalance.py`, a Python loop the owner started with the `balance` shell command. The legacy script stays in the repo for reference only.
- **Never edit or commit the owner's original folder** (`~/Desktop/Data/Desktop - Jonathan’s MacBook Air/BTC Balance Checker`). It holds the real `.env`, `address.txt` and logs. **Never commit real addresses, `.env` values or logs** (`constraints.md` C-001).
- The build came from an adversarial round: 3 Sonnet 5.5 builders, Codex as judge, entry A won 45/60 (`decisions.md` D-002).

## Evidence states

| State | Meaning |
|---|---|
| OBSERVED | Seen in the repo, a build, a test run or the live site. The source is named. |
| INFERRED | Implied by evidence but not checked directly. |
| PLANNED | Required by the brief or an accepted decision, not built yet. |
| PROPOSED | An agent's suggestion. Not binding. |
| UNKNOWN | No evidence. Write "Unknown" instead of guessing. |

Never promote INFERRED or PROPOSED to OBSERVED without new evidence.

## Roles

- **Claude (lead):** architecture, review, deploys, keeping `/ai/` true.
- **Sonnet 5.5 subagents (adversarial builders):** used once, for the build round.
- **Codex (judge and reviewer):** `codex exec`. Judged the round; its verdict drove the fix list (`changelog.md`).

## Change protocol

1. If the docs and the code disagree, the code wins. Fix the docs in the same task.
2. Any code change updates `project-state.md` and the docs it touches.
3. `src/` owns the shared UI files; `site/` copies are regenerated with `scripts/sync-ui.sh`, never hand-edited.
4. Before calling anything done, run `validation-checklist.md`.

## Routing

| Domain | Load |
|---|---|
| Rust backend, scanning, alerts | `project-overview.md`, `data-model.md`, `security.md` |
| Desktop or web UI | `components.md`, `interactions.md`, `design-system.md` |
| Landing page, copy | `pages.md`, `content.md`, `brand.md` |
| Releases, CI, Netlify | `deployment.md`, `environments.md` |
| Tests | `testing.md`, `validation-checklist.md` |
| Bugs | `known-issues.md` |
