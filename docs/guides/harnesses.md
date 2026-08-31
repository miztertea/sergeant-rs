# Harnesses, backends, and profiles

`sgt claude`, `sgt codex`, `sgt opencode`, `sgt goose`, and `sgt agy` launch Captain-facing harnesses from the estate root and pass trailing arguments through to the native program.

Actor backends execute workflow stages. The admitted actor backends are Claude, Codex, OpenCode, and Agy; `fake` deterministically exercises the loop without model usage, and Docker executes `execute`-kind stages rather than acting. A stage that declares `requires_ask = true` must route to a backend transport that supports asking — currently Claude and OpenCode's serve transport; Codex and Agy do not yet. Sergeant refuses unsupported combinations before creating Work side effects.

Profiles are named launch configuration in `sergeant.toml`. They may select a backend, model, permission mode, and supported backend options. Credentials remain owned by the native harness. Stage settings override Work defaults where the schema permits; resolved decisions are pinned for retries and recovery.

Permission and network options change authority and exposure. Use the least authority the Work needs and read the [profile reference](../reference/backends-and-profiles.md) before adding them.
