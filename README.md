# sergeant-rs

A Rust-native local agent execution surface: a single user daemon that accepts
durable Work, materializes git-worktree work surfaces, routes execution to
native agent harnesses (Claude first), records the complete execution
trajectory in an append-only journal, and exposes one loopback HTTP/SSE API
consumed by the CLI, TUI, and embedded HTML dashboard.

Clean-room successor to [Sergeant](https://github.com/miztertea/sergeant)
(Bash/tmux), informed by it rather than forked from it. The architecture it
prototypes is specified in
[`reference/proposal-depot-rust-execution-surface.md`](reference/proposal-depot-rust-execution-surface.md)
(where the product is called "Depot" — see the deviation register in
[`GAUNTLET.md`](GAUNTLET.md)).

## Status

Prototype under active construction via a gauntlet-loop development method:
per-milestone contracts in [`docs/gauntlet/contracts/`](docs/gauntlet/contracts/),
method in [`reference/notes/gauntlet-pattern.md`](reference/notes/gauntlet-pattern.md),
append-only build ledger in [`GAUNTLET.md`](GAUNTLET.md), running lessons in
[`LESSONS.md`](LESSONS.md).

Binary: `sgt`. Build with `cargo build`; gates are
`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.

## Layout

- `src/` — single crate (daemon, API, CLI, TUI, web, domain, runtime, backends)
- `docs/gauntlet/` — milestone contracts
- `reference/` — committed reference corpus (proposal, vendored Sergeant
  upstream at a pinned SHA, technique notes); evidence, not source
