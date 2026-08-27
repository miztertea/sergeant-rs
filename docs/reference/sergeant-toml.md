# `sergeant.toml` reference

The manifest is strict TOML; unknown fields are refused.

```toml
[estate]
name = "example"
default_backend = "claude"
default_workflow = "implement-change"
retention = 1000
# surfaces_dir = ".sergeant/data/surfaces"
# data_dir = ".sergeant/data"

[[repo]]
name = "app"
origin = "https://github.com/example/app.git"
instructions = "suppress"
# upstream = "https://github.com/upstream/app.git"

[[knowledge]]
name = "notes"
path = "/home/me/notes"
ignore = ["*.log", "drafts/**"]

[group.product]
repos = ["app"]
brief = "Repositories shipped as one product"
```

`[estate].name` is required. `default_backend`, `default_workflow`, `surfaces_dir`, `data_dir`, and `retention` are optional. Relative directories resolve from the estate root. Retention defaults to 1,000 and cannot be lower than 64.

Each `[[repo]]` requires a unique `name`. Its mount is always derived as `repos/<name>`; `path` is removed and refused. `origin`, `upstream`, and `instructions` are optional. Groups use `[group.<name>]`, require a `repos` array whose entries must each be a declared `[[repo]]` name (the array itself may be empty), and may carry a brief.

Each `[[knowledge]]` entry declares a local path read as **evidence**: a unique plain `name`, a required `path` (relative paths resolve from the estate root), an optional `ignore` array of globs, and an optional `context_fields` array of column names. A knowledge source is never a mount — nothing is cloned, no worktree is cut from it, and nothing writes to it. A path that resolves inside a repository mount, inside `surfaces_dir`, or inside `data_dir` is refused by name, because those are locations the estate itself mutates. `ignore` globs *extend* the built-in exclusion set (dotfiles, `.env*`, private keys, keystores, credential and secret files) and can never narrow it; excluded paths are reported as excluded rather than silently omitted.

`context_fields` governs a different boundary from `ignore`, at a different granularity. `ignore` decides which *bytes are read at all* and speaks in paths. `context_fields` decides which *values may leave a tabular dataset as text* and speaks in columns — because a CSV of support tickets is an ordinary knowledge source whose `email` column is not, and no path pattern can say that. **Its default is none.** A CSV, JSON or Parquet file under a knowledge source is registered as a dataset, read in place through a bounded canned query, counted and profiled in aggregate whether or not the key is present; without it, no row's text becomes a retrievable context unit. Narrowing the list later retracts what a wider one exposed: the declared columns are part of the reader's identity, so changing them supersedes the generation and removes its units.

Profiles are named launch records described in [backends and profiles](backends-and-profiles.md). The legacy `[workspace]` and `[[repository]]` vocabulary is refused with migration guidance.
