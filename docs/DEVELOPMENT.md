# Developing sergeant-rs

This is the dev rulebook: the working rules a session needs when it is
changing sergeant-rs's own code, tests, docs, or CI — as opposed to
`AGENTS.md`, which teaches a harness how to *use* `sgt` against any estate
repository. `AGENTS.md`'s "working on sergeant-rs itself" row points here
(clone-is-distro: the dev rulebook is repo content, not a separate product).
This file used to be `CLAUDE.md`; that path is now a git symlink to
`AGENTS.md` per the North Star ruling (`NORTH-STAR.md`, "In parallel — the
instrument"), and every rule that lived under the old `CLAUDE.md` moved here
unchanged in substance.

## Commands

```sh
cargo build                                          # debug build of the `sgt` binary
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
                                                     # the gates; all three must be green before any commit
cargo test --test m4_backends                        # one suite (m1_event_core … m10_harness)
cargo test --test m6_surfaces t5                     # one test by name substring
SERGEANT_CLAUDE_TESTS=1 cargo test --test m4_backends -- --ignored
                                                     # opt-in tests against the real `claude` CLI (bills tokens)
scripts/demo.sh                                      # §39 end-to-end walkthrough, fake backend, exits 0 or the walkthrough is broken
scripts/gate.sh "<intent>"                           # shipping gate via the no-mistakes pipeline (see below)
```

First build is slow: bundled DuckDB compiles ~500 C++ translation units (~10 min cold). `Cargo.toml` pins `[profile.dev.package.libduckdb-sys] debug = false` — removing it balloons `target/` from ~5 GB to ~15 GB. Never point an external pipeline's builds at this checkout's `CARGO_TARGET_DIR`: shared caches bake foreign `env!(CARGO_MANIFEST_DIR)` paths into reused test binaries (diagnosed 2026-08-09, see the ledger's M6 pause marker). The same hazard's other face: a disposable probe copy that shares this checkout's cache overwrites its binary slots — after any probe-copy build, rebuild the main checkout before measuring `target/debug/sgt` (bit twice, 2026-08-10, Bug Sprint 1 entry).

Build-dir placement, not just contamination, is its own hazard: disposable worktrees and their `CARGO_TARGET_DIR`s for probe and verify-agent work belong on disk-backed storage such as `/var/tmp/<name>`, never on tmpfs `/tmp` or a session scratchpad. On Cerberus, `/tmp` is a 16 GB tmpfs; on 2026-08-13 the WATCH build gauntlet's agent build dirs filled it, and every subsequent `Bash` invocation across every session on that host started failing — the harness's output-capture write hit `EDQUOT` while the command underneath still ran, so the box looked like a broken shell rather than a full disk (incident row and evidence in `docs/environments/cerberus.md`; #70). Clean up disposable build dirs after use instead of leaving them to accumulate.

## What this is

A local agent execution daemon (`sgt`): durable Work items run staged workflows in git-worktree surfaces, executed by native agent harnesses (Claude via headless `claude -p` turns; a deterministic fake backend for tests), with the entire trajectory event-sourced into an append-only journal. Specified by `reference/proposal-depot-rust-execution-surface.md` (which calls the product "Depot"); the N-series (ICM workflows, per-stage harnesses, Docker execute stages) is governed by its successor, `reference/proposal-next-iteration-icm-workflows.md`, kicked off by the N0 contract's rulings. Departures from either proposal live in GAUNTLET.md's deviation register and are settled — re-litigate one only by arguing its ruling is wrong, not by noticing the deviation exists. The product-level destination — what sergeant is *for*, past this repo's own build mechanics — is `NORTH-STAR.md`; this file stays scoped to how to work on the code.

## Architecture — the invariants that shape everything

- **The journal is the only truth.** Every state change is an event appended to a segmented, crash-tolerant JSONL journal (`src/runtime/journal.rs`; large payloads in a BLAKE3 content-addressed blob store). Everything else — the in-memory work projection, the DuckDB analytics tables (`src/runtime/analytics.rs`), the graph projection, every client screen — is a disposable projection rebuilt from it. Rebuild-on-start is the only population path; there is no snapshot loading (backlog B1 explains why).
- **One owner.** The daemon (`src/daemon.rs`) exclusively owns the data dir (daemon.lock) and all process handles. Clients hold no state and never touch storage.
- **Clients are equal.** The CLI and TUI (`src/tui.rs`) reach state only through the loopback HTTP/SSE API (`src/api.rs`) via `ApiClient`. This is enforced by tests, not convention: `tests/m6_surfaces.rs` t5 scans `tui.rs` for internals imports — widening that reach fails the test by design. If a client needs something the API lacks, extend the API. (The embedded dashboard, `src/web.rs`, used to be a third such client, held to the rule through its own `ApiViews` read surface — both are deleted, ADR 0011.)
- **Work state ≠ process state.** A Claude "session" is a durable conversation identity; the OS process exists per turn. Restart reconciliation (`src/runtime/recovery.rs`) resumes only on unambiguous evidence; ambiguity fails closed into `blocked` with a reason, never a guess.
- **Adjacent-append crash windows** are this architecture's recurring hazard (LESSONS L6): any path appending two causally-linked events must tolerate the second one missing or write one compound event. Check for this class in review of any journal-touching change.

Layout: single crate, lib + thin `main.rs` (`src/lib.rs` declares modules; integration tests need the lib target). `src/domain/` = types, `src/runtime/` = journal/projections/engine/recovery/router/analytics/graph, `src/backend/` = the §15 `Backend` trait + claude/fake/docker (docker runs `kind = "execute"` workflow stages; codex is a doc-stub per D6), `src/platform/` = the ADR 0002 platform boundary (`#[cfg]`-selected modules, not a trait — disk/data_dir/process facts, each with an UNVERIFIED macOS arm), `src/{api,cli,tui,daemon,harness,telemetry,watch}.rs` = surfaces (the embedded dashboard, `src/web.rs`, is deleted — ADR 0011; `harness` is the `sgt <harness>` compose-and-exec boundary, ADR 0006).

## Testing rules specific to this repo

- Tests live in per-milestone suites `tests/m1_event_core.rs` … `tests/m10_harness.rs` (the count is a smoke check, not a budget — re-measure with `cargo test` rather than trusting any prose figure; coverage baseline convention: `docs/perf/baseline-mvp-2026-08-12.md`). Suites that spawn daemons MUST go through `tests/support/mod.rs`'s `DataDir` guard — the `sgt(...)` helpers take `&DataDir` so an unreaped auto-spawned daemon is a type error, and the guard reaps by `/proc` argv scan on Drop. This exists because a measured leak accumulated ~89 orphan daemons in a day.
- A fix without a test that fails when the fix is reverted is not done (LESSONS L7). Every advertised backend capability flag needs a contract test against the installed harness (L8).
- The Claude adapter's behavior is *measured*, never assumed from docs — exit codes lie, `subtype` lies, model aliases silently substitute (L1). The version gate is pinned in `src/backend/claude.rs`; re-measure on any CLI version bump.
- **Code is code (R-S0-12).** Any diff that changes executable behavior — `src/`, `tests/`, `scripts/`, CI config, workflow `.js` — takes the full multi-axis loop (panel, adversarial verify, fix, adjudication). Measurement-template exemptions cover only phases that write no code; a builder's self-probe is panel input, never a substitute (L13).
- Tests run in two known environments with opposite constraints — design fault-injection fixtures for both or probe-gate with a loud `SKIPPED-ENV`: the root dev container (permission-bit tricks silently pass; EAGAIN≡EWOULDBLOCK; O_APPEND defeats post-start file sabotage) and GitHub's non-root 2-core runner (no `CAP_LINUX_IMMUTABLE`; O_DIRECT alignment unenforced). Locally-fixable preconditions stay hard failures; shapes no hosted-runner user can change skip loudly (GAUNTLET.md's S2 entry, issue #31 / Environmental behavior, has the run evidence).
- After running suites, `pgrep -f "debug/sgt [-]-data-dir"` should find nothing — that bracket is the whole trick: it makes the pattern non-self-matching (an unbracketed pattern matches your own shell; this bit two sessions). **The bracket belongs on every process-matching command, not just the checking ones.** With `pgrep` a self-match is a confusing extra row; with `pkill` it kills your own shell, and the failure is *silent* — the turn ends at exit 144 with no output, which reads as a hung command rather than an error. A 2026-08-14 session quoted this rule in a dispatch brief and then killed its own shell with an unbracketed `pkill` twice, because the rule was written about `pgrep` and got scoped to "checking" rather than to "matching processes."
- Test artifacts follow the same placement rule as build dirs: nothing a suite creates may be left in `std::env::temp_dir()`. On Cerberus that resolves to a 16 GB tmpfs, and a suite that leaks one file per run leaks it there (#108). Cleanup belongs in the code under test or an RAII guard — **not** in the body of the happy-path test, because the failure-path test is then the one that leaks.
- Commits that fully close an issue carry a `Fixes #NN` trailer; a PR body lists only closures whose commits are actually on the branch (Bug Sprint 1 / S2 precedent).

## Session conduct on this repo

These fold in the operating-discipline invariants adjudicated out of the 126
`agents-invariant` unit corpus (full disposition table:
`docs/icm/agents-invariant-dispositions.md`) that are specific to *developing*
sergeant-rs rather than to *using* `sgt` against an estate — the latter live
in `AGENTS.md` instead.

- Working directly on this repository in one session still requires the
  normal delivery discipline — tests, review, and the shipping gate — even
  though it runs in-session rather than as a dispatched Work item; the mode
  never waives them. <!-- BU-0018, BU-0113 -->
- Never push directly to a default branch; a session working here still goes
  through a branch and review like any other change. <!-- BU-0114 -->
- A workflow stage or actor executing inside a worktree never invokes
  `scripts/gate.sh`/no-mistakes itself — only the top-level orchestrating
  session owns a shipping-gate run, matching the single-owner posture the
  engine itself enforces on the data dir. <!-- BU-0041, BU-0122, BU-1196 -->
- A command's own `--help`, its emitted usage/error contract, and its tests
  are the authority over prose when they disagree; file an issue on the
  mismatch rather than trusting either doc silently. <!-- BU-0107 -->
- Secrets are never committed. Project/estate config files may contain paths
  but never credentials or tokens. <!-- BU-0055, BU-0259 -->
- A change to `.sergeant/workflows/` content (not just `src/`/`tests/`) still
  goes through review and the test suites that read it before landing.
  <!-- BU-0120 -->

## The development record (read before changing method or scope)

- **GAUNTLET.md** — append-only ledger: the deviation register, backlog rows with named triggers, per-milestone scorecards and adjudication rulings (current numbering lives in the file itself, not here — refer, don't copy). Append; never rewrite history.
- **LESSONS.md** — binding on development here. Highest-leverage: measure the claude CLI (L1), point fresh reviewers at the register first (L3), mutation probes only in disposable copies outside the tree (L5), keep fix commits separable from build commits (L10), re-read governing text at decision time — summaries are orientation, not authority (L12).
- **docs/gauntlet/contracts/** — the milestone contracts the code was built and reviewed against; `reference/notes/gauntlet-pattern.md` defines the loop **and the binding model spread** (Sonnet executes contracts, Opus judges outcomes, Fable is the one orchestrator seat and never fans out — dated revision 2026-08-10, ruling R-S0-13); `resources/` holds every orchestration script as run, per-series.
- Design decisions log their Ponytail rung (R1–R7 ladder in `reference/notes/ideaos-agent-contract.md`).
- `reference/notes/` are **living method docs**, revised in place with dated entries (their own convention — see the economy revisions). Everything else under `reference/` — the proposals, `sergeant-upstream/` — is frozen evidence: don't edit it to change behavior.

## Shipping gate

`scripts/gate.sh "<intent>"` runs the no-mistakes pipeline (`--skip push,pr,ci`; push/PR handled manually). It requires a clean tree, self-heals the pipeline daemon with the env the host needs (`IS_SANDBOX=1` only under root — it exists for root containers; the script is portable across the measured `docs/environments/` hosts), and gives the pipeline a private cargo cache. While a run is active the branch is pipeline-owned: don't commit locally until the run reaches an outcome, then `no-mistakes axi respond` on gates and `no-mistakes axi sync --recover` to take custody of pipeline commits.

**This paragraph is a summary, not the procedure. The procedure is `.sergeant/workflows/validate-and-ship/` — load it before driving a gate by hand.** Its `40-drive-gates` stage carries the full decision table over `branch_sync.next_action.code` (`sync` / `continue_active_run` / `recover_custody` / `blocked_recover_dirty` / `user_owned`) and the finding taxonomy that decides what an actor may authorize (`auto-fix`), what it merely records (`no-op`), and what only the user may rule on (`ask-user`, relayed verbatim); `50-reconcile-custody` owns custody return, including the `--keep-local` remediation for a dirty-worktree refusal. A 2026-08-14 session hand-rolled seven stages of this workflow across a full day and invented a workaround around `--keep-local` rather than using it, because this prose read as complete and it never reached the catalog (`LESSONS.md` L20). Two clarifications that prose has cost a session: pipeline-owned means **do not write to the worktree at all** — an untracked file is enough to block `--recover` — and undo on a gated branch is `git revert`, never `git reset`, because a pipeline that keeps its own repo of the branch rejects any backwards move as non-fast-forward.

Remote containers do not ship with no-mistakes — install it at session start. The release installer (`curl -fsSL https://raw.githubusercontent.com/kunchenguid/no-mistakes/main/docs/install.sh | sh`) 403s through this environment's proxy (release-API fetches are blocked); the working path is source build, the M1 precedent: `git clone --depth 1 https://github.com/kunchenguid/no-mistakes && go build -o ~/.cargo/bin/no-mistakes ./cmd/no-mistakes` (Go is preinstalled). Note the built version in the ledger entry that first uses it (M-series pinned v1.47.0; drift is a fact to record, not a blocker). If the gate genuinely cannot run, the fallback regime is R-S0-1 (orchestrator-verified gates + hygiene sweep), recorded per milestone — never silence.

## Environments

Repo invariants live in this file; **per-host facts live in `docs/environments/`** (one dated, evidence-cited file per environment — cloud container, GH runner, and each new host on first contact). Never assume another environment's facts apply: measure, then record there. Fixtures asserting environment facts probe-gate per the testing rules above.

Run `scripts/probe-env.sh` once at session start on **any** host, before doing anything else — it measures uid/DAC/CAP_LINUX_IMMUTABLE, disk and writable-allowance, O_DIRECT alignment behavior (open AND unaligned-write, the two-valued fact the journal's fault-injection fixtures actually branch on), network/proxy posture, Docker, and the claude/cargo/rustc toolchain, and prints them as a dated markdown table in the exact `docs/environments/` format — paste its output into that host's file. It never exits nonzero for an absent tool or negative result (only for its own bug); regression coverage lives in `scripts/probe-env-selftest.sh`, run with `bash scripts/probe-env-selftest.sh`.

## Remote-container operations

These sessions run in ephemeral containers that reset without warning (three resets in one S-series day; a reset also wipes installed tools and `target/`, costing a ~10-min cold DuckDB rebuild):

- **Push after every green gate.** Anything unpushed can vanish; everything pushed made the S2 resets free. In-flight workflows survive via cache-resume (`resumeFromRunId`) — orphaned agents' commits recover from git.
- **On wake or after any reset, restore before touching anything**: `git fetch origin <branch> && git checkout <branch> && git reset --hard origin/<branch>`. A freshly reset container silently gives you a stale clone of `main`; two S-session near-misses started by editing it.
- Re-install no-mistakes (above) and re-warm the build in the background while doing doc work.
