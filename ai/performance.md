# Performance

| Item | Measured |
|---|---|
| `site/` total | 152 KB including 2 woff2 fonts (OBSERVED, `du -sh site`) |
| JS | No framework; `site/app/app.js` 114 lines, `addr.js` 86 lines |
| macOS app | 4.22 MiB (`opt-level = "s"`, LTO, strip) |
| Fonts | Self hosted, `font-display: swap` |
