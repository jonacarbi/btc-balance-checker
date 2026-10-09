# Environments

Needs Rust stable, Node 22, and on Linux `libwebkit2gtk-4.1-dev`.

```
npm ci
npx tauri dev                 # desktop app
npx tauri build --bundles app # macOS .app
node tests/serve.mjs          # serve site/ locally for tests
```
