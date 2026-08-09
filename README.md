# sergeant-rs

A Rust-native local agent execution surface: a single user daemon that accepts
durable Work, materializes git-worktree work surfaces, routes execution to
native agent harnesses (Claude first), records the complete execution
trajectory in an append-only journal, and exposes one loopback HTTP/SSE API
consumed by the CLI, TUI, and embedded HTML dashboard — clients are equal;
every one of them is a projection of the same API.

Clean-room successor to [Sergeant](https://github.com/miztertea/sergeant)
(Bash/tmux), informed by it rather than forked from it. The architecture is
specified in
[`reference/proposal-depot-rust-execution-surface.md`](reference/proposal-depot-rust-execution-surface.md)
(where the product is called "Depot" — see the deviation register in
[`GAUNTLET.md`](GAUNTLET.md)).

## Status

**P0 prototype complete.** The §38 vertical slice — submit → worktree surface
→ staged workflow → native Claude execution → needs-input pause → respond →
completed, with the whole arc journaled, projected, and observable live — runs
end to end. 203 tests (plus 2 opt-in live-Claude tests) behind
`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.

## Try it

```sh
cargo build
scripts/demo.sh          # the §39 walkthrough: deterministic, fake backend,
                         # prints what happened and where the evidence lives
```

Real usage, from any git repository:

```sh
sgt run "fix the flaky retry test"   # submit work; auto-spawns the daemon,
                                     # materializes a worktree, starts the workflow
sgt status                           # daemon health and work counts
sgt work list                        # fleet at a glance
sgt work show <id>                   # one item: stage, execution, surface, events
sgt respond <id> "use exponential backoff"   # answer a paused, asking work item
sgt cancel <id> / sgt retry <id>
sgt analytics <question>             # canned §22 questions from the DuckDB projection
sgt                                  # no subcommand: the TUI (fleet + detail, live)
sgt web                              # print the dashboard URL, token included
sgt doctor                           # diagnose the install; every failure names its remedy
```

Every command takes `--json` for machine-readable output. Executions run on
the real `claude` CLI when routed to it (headless print-mode turns over a
durable session — see `src/backend/claude.rs`); the deterministic fake
backend covers tests and the demo.

## Architecture in one paragraph

The daemon owns everything; clients hold no state. Work is durable intent;
process state is disposable and reattachable. Every state change is an event
in a segmented, crash-tolerant JSONL journal (large payloads in a
content-addressed blob store), and everything else — the in-memory work
projection, the DuckDB analytics tables, the graph projection, every screen
of every client — is rebuilt from it. Ambiguity after a crash fails closed:
work blocks and says why rather than guessing. The design principles are §40
of the proposal; the regression catalog from Bash-Sergeant's failure modes is
`tests/m4_backends.rs`.

## Layout

- `src/` — single crate, one binary (`sgt`): daemon, HTTP/SSE API, CLI, TUI,
  web dashboard, domain types, runtime (journal, projections, engine,
  recovery, analytics, graph), backends (claude, fake)
- `tests/` — per-milestone acceptance suites (m1–m6) + shared test support
- `scripts/demo.sh` — the executable §39 walkthrough; `scripts/gate.sh` —
  shipping-gate runner
- `docs/gauntlet/contracts/` — the seven milestone contracts (M0–M6)
- `reference/` — committed reference corpus (proposal, vendored Sergeant
  upstream at a pinned SHA, method + technique notes); evidence, not source

## Development record

Built via a gauntlet-loop method: bounded contracts, blind multi-model critic
panels, adversarial verification of every finding, evidence-only
adjudication. The method definition is
[`reference/notes/gauntlet-pattern.md`](reference/notes/gauntlet-pattern.md);
the append-only build ledger with per-milestone scorecards, deviations
D1–D8, and rulings is [`GAUNTLET.md`](GAUNTLET.md); the lessons that now
bind the loop (L1–L10) are [`LESSONS.md`](LESSONS.md).
