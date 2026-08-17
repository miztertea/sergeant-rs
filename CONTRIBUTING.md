# Contributing

## Build

Requires Rust (edition 2024) and `git`.

```sh
cargo build
```

First build is slow — bundled DuckDB compiles from source. See
[`README.md`](README.md#get-it) for the current time estimate.

## Test

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

All three must be green before a commit. `cargo test --test <name>` runs one
suite, `cargo test --test <name> <substring>` runs one test.

## Pull requests

`main` is protected: changes land through a pull request, not a direct push.
CI (`fmt`, `clippy`, `cargo test`, ShellCheck, a macOS compile check, and
dependency policy) runs on every PR and must pass.

## Where the deeper rules live

This file is deliberately small. The working rules for changing this repo's
code, tests, docs, or CI — architecture invariants, testing conventions,
session conduct, environment quirks — live in
[`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md). Read that before your first
non-trivial change.
