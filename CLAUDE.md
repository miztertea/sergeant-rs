# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build                                          # debug build of the `sgt` binary
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
                                                     # the gates; all three must be green before any commit
cargo test --test m4_backends                        # one suite (m1_event_core … m6_surfaces)
cargo test --test m6_surfaces t5                     # one test by name substring
SERGEANT_CLAUDE_TESTS=1 cargo test --test m4_backends -- --ignored
                                                     # opt-in tests against the real `claude` CLI (bills tokens)
scripts/demo.sh                                      # §39 end-to-end walkthrough, fake backend, exits 0 or the walkthrough is broken
scripts/gate.sh "<intent>"                           # shipping gate via the no-mistakes pipeline (see below)
```

First build is slow: bundled DuckDB compiles ~500 C++ translation units (~10 min cold). `Cargo.toml` pins `[profile.dev.package.libduckdb-sys] debug = false` — removing it balloons `target/` from ~5 GB to ~15 GB. Never point an external pipeline's builds at this checkout's `CARGO_TARGET_DIR`: shared caches bake foreign `env!(CARGO_MANIFEST_DIR)` paths into reused test binaries (diagnosed 2026-08-09, see the ledger's M6 pause marker). The same hazard's other face: a disposable probe copy that shares this checkout's cache overwrites its binary slots — after any probe-copy build, rebuild the main checkout before measuring `target/debug/sgt` (bit twice, 2026-08-10, Bug Sprint 1 entry).

## What this is

A local agent execution daemon (`sgt`): durable Work items run staged workflows in git-worktree surfaces, executed by native agent harnesses (Claude via headless `claude -p` turns; a deterministic fake backend for tests), with the entire trajectory event-sourced into an append-only journal. Specified by `reference/proposal-depot-rust-execution-surface.md` (which calls the product "Depot"); the N-series (ICM workflows, per-stage harnesses, Docker execute stages) is governed by its successor, `reference/proposal-next-iteration-icm-workflows.md`, kicked off by the N0 contract's rulings. Departures from either proposal live in GAUNTLET.md's deviation register and are settled — re-litigate one only by arguing its ruling is wrong, not by noticing the deviation exists.

## Architecture — the invariants that shape everything

- **The journal is the only truth.** Every state change is an event appended to a segmented, crash-tolerant JSONL journal (`src/runtime/journal.rs`; large payloads in a BLAKE3 content-addressed blob store). Everything else — the in-memory work projection, the DuckDB analytics tables (`src/runtime/analytics.rs`), the graph projection, every client screen — is a disposable projection rebuilt from it. Rebuild-on-start is the only population path; there is no snapshot loading (backlog B1 explains why).
- **One owner.** The daemon (`src/daemon.rs`) exclusively owns the data dir (daemon.lock) and all process handles. Clients hold no state and never touch storage.
- **Clients are equal.** The CLI, TUI (`src/tui.rs`), and embedded dashboard (`src/web.rs`) reach state only through the loopback HTTP/SSE API (`src/api.rs`) via `ApiClient`/`ApiViews`. This is enforced by tests, not convention: `tests/m6_surfaces.rs` t5 scans for internals imports AND pins the exact public method set of `ApiViews` — widening that surface with a non-endpoint method fails the test by design. If a client needs something the API lacks, extend the API.
- **Work state ≠ process state.** A Claude "session" is a durable conversation identity; the OS process exists per turn. Restart reconciliation (`src/runtime/recovery.rs`) resumes only on unambiguous evidence; ambiguity fails closed into `blocked` with a reason, never a guess.
- **Adjacent-append crash windows** are this architecture's recurring hazard (LESSONS L6): any path appending two causally-linked events must tolerate the second one missing or write one compound event. Check for this class in review of any journal-touching change.

Layout: single crate, lib + thin `main.rs` (`src/lib.rs` declares modules; integration tests need the lib target). `src/domain/` = types, `src/runtime/` = journal/projections/engine/recovery/router/analytics/graph, `src/backend/` = the §15 `Backend` trait + claude/fake (codex is a doc-stub per D6), `src/{api,cli,tui,web,daemon,telemetry}.rs` = surfaces.

## Testing rules specific to this repo

- Tests live in per-milestone suites `tests/m1_event_core.rs` … `tests/m6_surfaces.rs` (379 total + 4 opt-in as of the invariants/test-honesty round-2 fixer, 2026-08-11 — coverage 94.63% lines measured at S2 close by the committed convention, `docs/coverage/baseline-2026-08-10.md`; the count is a smoke check on this line, not a budget, and TH-R2-03 confirmed round 1's TH-9 fix here had already gone stale again by the very next diff — re-measure with `cargo test` rather than trusting this line). Suites that spawn daemons MUST go through `tests/support/mod.rs`'s `DataDir` guard — the `sgt(...)` helpers take `&DataDir` so an unreaped auto-spawned daemon is a type error, and the guard reaps by `/proc` argv scan on Drop. This exists because a measured leak accumulated ~89 orphan daemons in a day.
- A fix without a test that fails when the fix is reverted is not done (LESSONS L7). Every advertised backend capability flag needs a contract test against the installed harness (L8).
- The Claude adapter's behavior is *measured*, never assumed from docs — exit codes lie, `subtype` lies, model aliases silently substitute (L1). The version gate is pinned in `src/backend/claude.rs`; re-measure on any CLI version bump.
- **Code is code (R-S0-12).** Any diff that changes executable behavior — `src/`, `tests/`, `scripts/`, CI config, workflow `.js` — takes the full multi-axis loop (panel, adversarial verify, fix, adjudication). Measurement-template exemptions cover only phases that write no code; a builder's self-probe is panel input, never a substitute (L13).
- Tests run in two known environments with opposite constraints — design fault-injection fixtures for both or probe-gate with a loud `SKIPPED-ENV`: the root dev container (permission-bit tricks silently pass; EAGAIN≡EWOULDBLOCK; O_APPEND defeats post-start file sabotage) and GitHub's non-root 2-core runner (no `CAP_LINUX_IMMUTABLE`; O_DIRECT alignment unenforced). Locally-fixable preconditions stay hard failures; shapes no hosted-runner user can change skip loudly (GAUNTLET.md's S2 entry, issue #31 / Environmental behavior, has the run evidence).
- After running suites, `pgrep -f "debug/sgt [-]-data-dir"` should find nothing — that bracket is the whole trick: it makes the checking command non-self-matching (an unbracketed pattern matches your own shell; this bit two sessions).
- Commits that fully close an issue carry a `Fixes #NN` trailer; a PR body lists only closures whose commits are actually on the branch (Bug Sprint 1 / S2 precedent).

## The development record (read before changing method or scope)

- **GAUNTLET.md** — append-only ledger: the deviation register, backlog rows with named triggers, per-milestone scorecards and adjudication rulings (current numbering lives in the file itself, not here — refer, don't copy). Append; never rewrite history.
- **LESSONS.md** — binding on development here. Highest-leverage: measure the claude CLI (L1), point fresh reviewers at the register first (L3), mutation probes only in disposable copies outside the tree (L5), keep fix commits separable from build commits (L10), re-read governing text at decision time — summaries are orientation, not authority (L12).
- **docs/gauntlet/contracts/** — the milestone contracts the code was built and reviewed against; `reference/notes/gauntlet-pattern.md` defines the loop **and the binding model spread** (Sonnet executes contracts, Opus judges outcomes, Fable is the one orchestrator seat and never fans out — dated revision 2026-08-10, ruling R-S0-13); `resources/` holds every orchestration script as run, per-series.
- Design decisions log their Ponytail rung (R1–R7 ladder in `reference/notes/ideaos-agent-contract.md`).
- `reference/notes/` are **living method docs**, revised in place with dated entries (their own convention — see the economy revisions). Everything else under `reference/` — the proposals, `sergeant-upstream/` — is frozen evidence: don't edit it to change behavior.

## Shipping gate

`scripts/gate.sh "<intent>"` runs the no-mistakes pipeline (`--skip push,pr,ci`; push/PR handled manually). It requires a clean tree, self-heals the pipeline daemon with the `IS_SANDBOX=1` env this container needs, and gives the pipeline a private cargo cache. While a run is active the branch is pipeline-owned: don't commit locally until the run reaches an outcome, then `no-mistakes axi respond` on gates and `no-mistakes axi sync --recover` to take custody of pipeline commits.

Remote containers do not ship with no-mistakes — install it at session start. The release installer (`curl -fsSL https://raw.githubusercontent.com/kunchenguid/no-mistakes/main/docs/install.sh | sh`) 403s through this environment's proxy (release-API fetches are blocked); the working path is source build, the M1 precedent: `git clone --depth 1 https://github.com/kunchenguid/no-mistakes && go build -o ~/.cargo/bin/no-mistakes ./cmd/no-mistakes` (Go is preinstalled). Note the built version in the ledger entry that first uses it (M-series pinned v1.47.0; drift is a fact to record, not a blocker). If the gate genuinely cannot run, the fallback regime is R-S0-1 (orchestrator-verified gates + hygiene sweep), recorded per milestone — never silence.

## Environments

Repo invariants live in this file; **per-host facts live in `docs/environments/`** (one dated, evidence-cited file per environment — cloud container, GH runner, and each new host on first contact). Never assume another environment's facts apply: measure, then record there. Fixtures asserting environment facts probe-gate per the testing rules above.

Run `scripts/probe-env.sh` once at session start on **any** host, before doing anything else — it measures uid/DAC/CAP_LINUX_IMMUTABLE, disk and writable-allowance, O_DIRECT alignment behavior (open AND unaligned-write, the two-valued fact the journal's fault-injection fixtures actually branch on), network/proxy posture, Docker, and the claude/cargo/rustc toolchain, and prints them as a dated markdown table in the exact `docs/environments/` format — paste its output into that host's file. It never exits nonzero for an absent tool or negative result (only for its own bug); regression coverage lives in `scripts/probe-env-selftest.sh`, run with `bash scripts/probe-env-selftest.sh`.

## Remote-container operations

These sessions run in ephemeral containers that reset without warning (three resets in one S-series day; a reset also wipes installed tools and `target/`, costing a ~10-min cold DuckDB rebuild):

- **Push after every green gate.** Anything unpushed can vanish; everything pushed made the S2 resets free. In-flight workflows survive via cache-resume (`resumeFromRunId`) — orphaned agents' commits recover from git.
- **On wake or after any reset, restore before touching anything**: `git fetch origin <branch> && git checkout <branch> && git reset --hard origin/<branch>`. A freshly reset container silently gives you a stale clone of `main`; two S-session near-misses started by editing it.
- Re-install no-mistakes (above) and re-warm the build in the background while doing doc work.
