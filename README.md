# sergeant-rs

**A local execution surface for coding agents — durable work, isolated worktrees, and a complete evidence trail, behind one daemon.**

[![CI](https://github.com/miztertea/sergeant-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/miztertea/sergeant-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Terminal agent sessions are ephemeral: close the laptop and the conversation, the branch state, and the story of what happened scatter across scrollback, tmux panes, and your memory. sergeant-rs makes **work** the durable unit instead of the terminal. You submit an intent; a single user daemon materializes an isolated git worktree, drives a staged workflow on a native agent harness (Claude first), and records every state change in an append-only journal. Any client — CLI, TUI, web dashboard — can attach, observe live, answer the agent's questions, and walk away again. The daemon owns the work; the terminal is just a window onto it.

## See it

| TUI — work detail with live journal tail | Dashboard — fleet |
|---|---|
| ![TUI work detail](docs/img/tui-detail.png) | ![Dashboard fleet](docs/img/dashboard-fleet.png) |

| TUI — fleet | Dashboard — work detail |
|---|---|
| ![TUI fleet](docs/img/tui-fleet.png) | ![Dashboard work detail](docs/img/dashboard-work-detail.png) |

## Why it exists

Running one agent in one terminal works. Running *several*, across repos, over hours, does not:

- **Work outlives processes.** An agent "session" here is a durable conversation identity; the OS process exists per turn. Kill the daemon mid-execution and restart it — unambiguous evidence resumes the work, ambiguity fails closed into `blocked` with a stated reason. Never a guess, never a duplicate execution.
- **Evidence, not vibes.** Every transition — submit, worktree binding, stage entry, each model turn's raw output, the question the agent asked, your answer — is an event in a crash-tolerant journal (large payloads in a content-addressed blob store). "The agent did X" is a query, not a recollection.
- **Isolation by construction.** Each work item gets its own git worktree on its own branch (`sergeant/<work-id>`), outside your checkout. Agents never fight over your working tree or each other's.
- **One API, equal clients.** The CLI, the ratatui TUI, and the embedded HTML dashboard all consume the same loopback HTTP/SSE API. Nothing has a private shortcut — a structural test fails if one appears.
- **Analytics on your own history.** An embedded DuckDB projection (rebuilt from the journal, always disposable) answers questions like time-blocked-per-work; a graph projection exposes the work→stage→execution→event structure with journal provenance on every edge.

This is a clean-room Rust successor to [callmeradical/sergeant](https://github.com/callmeradical/sergeant) — the Bash/tmux original whose failure modes became this project's regression-test catalog. Sergeant's own inspiration was [kunchenguid/firstmate](https://github.com/kunchenguid/firstmate). Lineage matters here: the architecture is specified in [a full proposal](reference/proposal-depot-rust-execution-surface.md) committed to this repo, and every deviation from it is registered with rationale in [GAUNTLET.md](GAUNTLET.md).

## Quickstart

Requires Rust (edition 2024), `git`, and — for real agent execution — an installed [`claude` CLI](https://claude.com/claude-code). Everything below the demo also works without `claude` via the deterministic fake backend.

```sh
git clone https://github.com/miztertea/sergeant-rs
cd sergeant-rs
cargo build --release        # first build compiles bundled DuckDB (~10 min); after that it's fast

scripts/demo.sh              # the full walkthrough, deterministic, no tokens spent
```

The demo drives the entire loop in a throwaway repo — submit → worktree → stage runs and *stops to ask a question* → you answer → independent review stage → completed → retired — and prints where the evidence lives at every step (journal path, blob refs, analytics query, graph endpoint, dashboard URL). It exits 0 or the walkthrough is broken.

## Usage

From any git repository:

```sh
sgt run "add retry handling to the settlement worker"   # submit; the daemon auto-spawns if needed
sgt status                        # daemon health, work counts
sgt work list                     # the fleet
sgt work show <id>                # one item: stage, execution, surface, recent events
sgt respond <id> "yes, 3 attempts with backoff"         # answer a paused work item
sgt retry <id> · sgt cancel <id>
sgt analytics <question>          # canned analytical queries over your history
sgt                               # no subcommand: the TUI — fleet + detail, live over SSE
sgt web                           # print/open the dashboard URL (tokenized, loopback-only)
sgt doctor                        # diagnose the install; every failing check names its remedy
```

Every command takes `--json`. Workflows are content, not code: drop a `.sergeant/workflows/<name>/workflow.toml` with ordered stages and per-stage `CONTEXT.md` files in your repo, and route work to it. Backends are selected per work item (`--backend claude|fake`) or by named routing profiles in `sergeant.toml`.

## How it holds together

```
   sgt CLI ──┐
   sgt TUI ──┤            ┌────────────────────────── daemon (one per user) ──┐
   dashboard ┴─ HTTP/SSE ─┤  engine → workflows → routing → Backend trait     │
                          │      │                    ├── claude (headless    │
                          │  append-only journal      │     print-mode turns) │
                          │  + blob store             └── fake (deterministic)│
                          │      │                                            │
                          │  projections: in-memory · DuckDB · graph          │
                          └──── work surfaces: git worktrees, one per work ───┘
```

The journal is the single source of truth; every projection can be deleted and rebuilt from it (measured at ~15k events/s). The Claude adapter drives headless `claude -p` turns over a daemon-chosen durable session ID — behavior *measured* against the installed CLI, never assumed from docs, with a version gate and contract tests that re-measure on bumps. OpenTelemetry export (traces + metrics) is built in and off by default.

## Status

**P0 complete** (the full vertical slice above, 203 tests). Currently in **P1: performance baselining** — load/stress scenarios, resource measurement, and a public issue backlog of everything that shakes out; the TUI and dashboard get a dedicated usability phase after that. The prototype was built end-to-end by a multi-agent gauntlet loop (bounded contracts, blind critic panels, adversarial verification); the complete development record — including every wrong turn — is in [GAUNTLET.md](GAUNTLET.md) and [LESSONS.md](LESSONS.md), and the method in [reference/notes/gauntlet-pattern.md](reference/notes/gauntlet-pattern.md).

## Developing

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test   # the gates
cargo test --test m4_backends              # one suite
SERGEANT_CLAUDE_TESTS=1 cargo test --test m4_backends -- --ignored   # opt-in: real claude CLI, bills tokens
```

See [CLAUDE.md](CLAUDE.md) for the repo's working rules (they bind humans too) and `docs/gauntlet/contracts/` for what each milestone promised.

## License

[MIT](LICENSE). Inspired by [callmeradical/sergeant](https://github.com/callmeradical/sergeant), which was inspired by [kunchenguid/firstmate](https://github.com/kunchenguid/firstmate).
