# Design system

Source: `src/ledger.css` (OBSERVED).

| Token | Light | Dark |
|---|---|---|
| `--paper` | #f3f0e8 | #121211 |
| `--ink` | #141414 | #efece4 |
| `--mute` | #55524b | #a5a198 |
| `--hair` | #c9c4b6 | #3a3935 |
| `--accent` | #c33207 | #ff6a3d |
| `--ok` | #14653a | #5fd08f |
| `--err` | #a8201a | #ff8a80 |

Type: `--sans` Schibsted Grotesk 400 to 800, `--mono` Spline Sans Mono 400 to 600 for figures and addresses. Sizes `--t-small` 0.8125rem, `--t-base` 0.9375rem, `--t-figure` clamp(2rem, 1.2rem + 4vw, 4.25rem). Spacing `--s1` to `--s5`: 0.25, 0.5, 1, 1.5, 3 rem. Motion `--dur` 150ms.
Theme: follows `prefers-color-scheme`, overridable with `data-theme`.
