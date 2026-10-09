# Dependencies

| Package | Why |
|---|---|
| tauri 2 | Native shell and installers |
| tauri-plugin-dialog | Export save dialog |
| tauri-plugin-single-instance | One running instance, protects seen set |
| serde, serde_json | Settings and API JSON |
| ureq | Blocking HTTP, small |
| lettre (rustls) | SMTP alerts |
| bs58 (check), bech32 0.11 | Address checksums |
| dirs | OS data and config dirs |
| @tauri-apps/cli 2, @playwright/test | Build and browser tests |
