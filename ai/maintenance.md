# Maintenance

**Trigger: release a new version.** Bump `version` in `package.json` and `src-tauri/tauri.conf.json` and `Cargo.toml`; commit; `git tag vX.Y.Z && git push --tags`. CI attaches installers to the GitHub Release; the landing page reads them live.

**Trigger: site change.** Edit `site/` (or `src/` then `sh scripts/sync-ui.sh`); `netlify deploy --prod --no-build --dir site --site 8ed47e23-8fe9-4eea-aefc-b7fe2d8a20a2`.

**Trigger: shared UI change.** Edit `src/ledger.css`, run `scripts/sync-ui.sh`, commit both.

**Trigger: API breaks.** Parsing lives in `src-tauri/src/api.rs` (desktop) and `site/app/app.js` (web). Add a test with the new response shape first.
