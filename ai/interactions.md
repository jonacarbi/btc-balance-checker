# Interactions

- Scan: blockchain.info `balance?active=a|b|c` in batches; on error, mempool.space per address. HTTP 429 honours `Retry-After`, else backoff 2 s doubling to 60 s; success resets.
- First seen alert: an address alerts once; seen set persists. If the seen write fails, the alert still sends and the UI warns it may repeat.
- Watch mode: problems from each scan are emitted as `scan-problems` and shown in the report line.
- Export: save dialog (`tauri-plugin-dialog`); cancel is silent.
- Second launch focuses the running window (`tauri-plugin-single-instance`).
- Web app: 10 s fetch timeout; non finite numbers show "unexpected response"; controls re-enable in `finally`.
