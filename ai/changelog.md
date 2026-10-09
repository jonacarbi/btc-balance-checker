# Changelog

- 2026-10-09: Landing hero rewritten to "Your address list, checked every minute." (owner: old line read as AI slop).
- 2026-10-09: Tagged v2.0.0; CI published 6 installers to the GitHub Release.
- 2026-10-09: Created `/ai/` layer after deploy. Replaced real sample addresses in `tests/` with public ones; removed the hardcoded recipient email from `legacy/JCBalance.py` (now `BTC_ALERT_TO`).
- 2026-10-09: Entry A applied the Codex fix list: watch errors surface, write failures propagate, Windows console attach, 429 backoff, web fetch timeout, single instance, `create_new` temp files. Ported B's save dialog and test alert, C's isolated `--cli` with `--once` and static download fallbacks.
- 2026-10-09: Codex judged the build round: A 45, C 44, B 43 (of 60). Verdict listed 6 bugs in A plus a shared temp file permission flaw.
- 2026-10-09: Three Sonnet 5.5 builders built competing entries from one spec.
