# Validation checklist

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` passes
- [ ] `node --test tests/*.test.mjs` passes
- [ ] `npx playwright test` passes (live test: `LIVE=1 npx playwright test -g live`)
- [ ] `sh scripts/sync-ui.sh && git diff --exit-code site/` shows no drift
- [ ] `npx tauri build --bundles app` builds
- [ ] `git grep` finds no real addresses, emails or tokens
- [ ] Site returns 200 for `/`, `/app/`, `/llms.txt`, `/index.md`
- [ ] `project-state.md` and `changelog.md` updated
