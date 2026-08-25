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

[group.product]
repos = ["app"]
brief = "Repositories shipped as one product"
```

`[estate].name` is required. `default_backend`, `default_workflow`, `surfaces_dir`, `data_dir`, and `retention` are optional. Relative directories resolve from the estate root. Retention defaults to 1,000 and cannot be lower than 64.

Each `[[repo]]` requires a unique `name`. Its mount is always derived as `repos/<name>`; `path` is removed and refused. `origin`, `upstream`, and `instructions` are optional. Groups use `[group.<name>]`, require a `repos` array whose entries must each be a declared `[[repo]]` name (the array itself may be empty), and may carry a brief.

Profiles are named launch records described in [backends and profiles](backends-and-profiles.md). The legacy `[workspace]` and `[[repository]]` vocabulary is refused with migration guidance.
