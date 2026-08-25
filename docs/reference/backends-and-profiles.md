# Backends and profiles

| Backend | Actor execution | Asking stage | Notes |
|---|---:|---:|---|
| Claude | yes | yes | native harness owns credentials |
| Codex | yes | no | asking stages are refused |
| OpenCode | yes | yes through supported serve transport | native configuration applies |
| Antigravity (`agy`) | yes | no | one OS process per turn; no export/history verb |
| fake | deterministic test | deterministic | no model spend |
| Docker | execute stages | n/a | network currently `none` |

Goose has a launch surface (`sgt goose`) but is not admitted as an actor backend in v0.2.4 — it is a passthrough exec only, with no `Backend` adapter behind it.

Profiles are declared in the estate manifest and can carry their name, backend, optional executable override, config home, environment overrides, default model, and backend-specific options. Supported permission-mode values are mechanically closed by the profile parser; use `sgt doctor` or command help to reject unsupported values rather than guessing. Codex network access is an explicit profile option. A profile never stores or supplies harness credentials.
