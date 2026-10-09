# Deployment

| Target | Detail |
|---|---|
| Site | Netlify project `btc-balance-checker`, id `8ed47e23-8fe9-4eea-aefc-b7fe2d8a20a2`, https://btc-balance-checker.netlify.app, publish dir `site`, no build step |
| Repo | https://github.com/jonacarbi/btc-balance-checker, description carries the site URL |
| Installers | `.github/workflows/build.yml`: macOS universal dmg, Windows exe + msi, Linux deb + rpm + AppImage; attached to the GitHub Release on `v*` tags |

Deploy site: `netlify deploy --prod --no-build --dir site --site 8ed47e23-8fe9-4eea-aefc-b7fe2d8a20a2`.
