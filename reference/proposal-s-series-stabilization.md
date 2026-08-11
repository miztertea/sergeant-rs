# Sergeant-rs S-Series — Coverage, Stabilization, and Repo Hygiene

Proposal, 2026-08-10, as amended at S0 adjudication (challenge record:
`docs/gauntlet/notes/s0-adjudication.md`; rulings:
`docs/gauntlet/contracts/S0.md`). Companion to
`proposal-depot-rust-execution-surface.md` (the P0 spec) and
`proposal-next-iteration-icm-workflows.md` (Program A/B, the N-series). This
program is orthogonal to both: it changes no product behavior and adds no
execution semantics. Its subject is the *trustworthiness of the quality
claims* the ledger already makes — measured coverage, pinned flake behavior,
honest CI, and a repository whose surface reflects what it is.

Sections are numbered for contract citation (§N), per house convention.

# 1. Mission

Make the test suite's protection measurable, close the gaps worth closing,
and keep both without slowing prototyping. Deliverables are a rerunnable
coverage harness whose exact measurement convention is published with its
first number (L11), a dated baseline with a known-losses register, a bounded
test-only remediation pass, and a non-blocking CI coverage lane — each landed
through the S-series gate regime (§15.1) with its ledger entry.

# 2. Why now

- P0 closed with 218 tests + 2 opt-in and gates green at every milestone, but
  no line of coverage has ever been measured; the suites' protection is
  asserted by review, not by instrument.
- The backlog already names coverage gaps as issues (#20 crash-point
  injection, #22 workspace edge cases); a baseline turns "we suspect" into
  "we measured" before any of them is taken up. (#21, dashboard JS, is *not*
  in this instrument's reach — see §13.)
- The N-series (PR #27) is content-only until N3 and touches no `src/`,
  `tests/`, `scripts/`, `Cargo.toml`, or `.github/` — this window is the
  cheapest this work will ever be, with zero code-level collision risk.
- GitHub classifies the repo as majority Shell because `reference/`'s vendored
  evidence tree outweighs the crate (§9); the fix is one `.gitattributes`.
- The sister project nim-proxy (miztertea/nim-proxy) provides measured
  precedent: a daemon-spawning Rust crate at ~96% line coverage via
  cargo-llvm-cov, including the subprocess-coverage mechanics this repo needs
  (LLVM_PROFILE_FILE inheritance, SIGTERM-before-SIGKILL teardown), and a
  documented cost lesson this program must not repeat (§10).

# 3. Doctrine

1. **Measure first** (LESSONS L1 generalized): the baseline runs before any
   remediation is scoped; remediation cites measured gaps by issue number.
   Tool behaviors asserted in this proposal (profile handling, profraw
   semantics, `#[cfg(test)]` exclusion, flag meanings) are *hypotheses the
   S1 builder verifies against the installed cargo-llvm-cov*, not facts.
2. **YAGNI-first coverage** (nim-proxy v0.6.3 evidence): before adding a test
   seam to reach a branch, prove it unreachable via an in-module
   `#[cfg(test)]` poke or an existing black-box trick. Production code changes
   made solely for testability are R7 machinery requiring rung justification.
3. **Residual gaps are documented, not chased**: every uncovered region either
   gets a test, an issue, or a written reason in the baseline doc. The gate
   number never implies completeness.
4. **A coverage instrument that lies is worse than none**: known-loss sites
   are registered (§6), corrupt or discarded profraws are counted and
   reported (§6.3), and under-reporting is never misread as a gap.
5. **LEGO-brick tooling**: standard tools (cargo-llvm-cov, rustup llvm-tools,
   GitHub Actions) composed as shipped, versions recorded and drift-guarded;
   no bespoke coverage machinery (Ponytail R5 before R7).

# 4. Program shape

| Milestone | Outcome | Template |
|---|---|---|
| **S0** | Adjudication only: the challenge round run, every finding ruled, proposal amended, rulings R-S0-1… recorded | N0 precedent |
| **S1** | Coverage baseline in three strictly ordered phases (§4.1); repo hygiene lands in phase 1 | P1-PERF (measurement) |
| **S2** | Stabilization: test-only remediation waves against S1's issues; every new test falsifiable per L7; residual gaps written down; closes with the CI coverage lane as a rung-logged commit (one `workflow_dispatch` run proves it green) | M-contract, narrow |

S3 was demoted from milestone to S2 close-out commit at S0 (ruling R-S0-9:
its whole deliverable is one workflow file plus one number — an M-contract
around it would be R7 process machinery). The S2 contract is drawn only after
S1's baseline exists — its scope is S1's output, cited by issue number, never
restated (house rule).

## 4.1 S1 internal ordering (ruling R-S0-4)

The baseline is never measured against a moving tree:

1. **Phase 1 — instrument & hygiene.** Instrument repairs (§8.1), hygiene
   files (§9), CI trigger fix (§10), harness under `scripts/coverage/`, issue
   template. Committed; gates green; SHA recorded.
2. **Phase 2 — measurement.** All coverage and census runs execute against
   phase 1's SHA with the tree frozen. An instrument defect discovered here
   is fixed, committed, and phase 2 restarts from its beginning.
3. **Phase 3 — publication.** Baseline doc, issues, ledger entry. No code.

# 5. Tooling and the measurement convention

`cargo-llvm-cov` (installed: 0.8.7, recorded in the baseline) on the repo's
stable toolchain. Alternatives were assessed and rejected: tarpaulin
(ptrace-based; historically weak on forked/detached children — exactly this
repo's shape), grcov (same LLVM backend, extra dependency, no gain),
hand-rolled `-C instrument-coverage` (kept as the documented fallback).
Branch coverage is nightly-only and out of scope; the program reports
line/region/function coverage. The system LLVM (18) cannot read
rustc-21-era profraw — only the rustup `llvm-tools` component's tools are
used.

**Toolchain drift guard (R-S0-2):** `rust-toolchain.toml` floats on
`channel = "stable"` and stays that way (freezing it would change builds for
every program in the repo). Instead the harness records `rustc -vV` and the
cargo-llvm-cov version into the run's artifact dir at start, and refuses to
merge or compare profdata across differing recorded versions. The baseline
doc quotes both.

**The convention (R-S0-3, the L11 clause):** the exact command lines below
are committed in `scripts/coverage/` and quoted verbatim in the baseline
doc. A number produced any other way is not this program's number.

```sh
# per-suite collection, staged (dev profile — the same profile the repo's
# own gates run; diverges from P1-PERF's release rule deliberately, because
# coverage measures what `cargo test` exercises, not what ships)
cargo llvm-cov --no-report --lib
cargo llvm-cov --no-report --test m1_event_core
cargo llvm-cov --no-report --test m4_backends
cargo llvm-cov --no-report --test m3_execution
cargo llvm-cov --no-report --test m5_projections
cargo llvm-cov --no-report --test m2_daemon_api
cargo llvm-cov --no-report --test m6_surfaces

# single report over the pooled profdata
cargo llvm-cov report --summary-only
cargo llvm-cov report --lcov --output-path <artifacts>/lcov.info
cargo llvm-cov report --html
```

Convention constants, stated once and committed: target dir is
cargo-llvm-cov's default (`target/llvm-cov-target` — never `target/`, never
any shared cache, per CLAUDE.md's thrice-bitten rule); the two `#[ignore]`d
real-Claude tests are *not* run (recorded, §6.2); report scope is `src/**`
(`tests/`, `web/`, `reference/` excluded from the *report* — the builder
verifies and records the exact exclusion mechanism and cargo-llvm-cov's
actual `#[cfg(test)]` handling rather than trusting docs, per doctrine 1);
profraw accounting per §6.3. The S1 builder measures each constant's actual
behavior and the harness records `cargo llvm-cov show-env` output with every
run.

# 6. Subprocess coverage — the measured mechanics

The crate satisfies the preconditions (verified twice: static assessment and
the S0 challenge round):

- All spawning suites locate the binary via `env!("CARGO_BIN_EXE_sgt")`;
  nothing hardcodes `target/debug`. The demo test pins `SGT_BIN` explicitly,
  so `scripts/demo.sh` exercises the instrumented binary too.
- No spawn path calls `env_clear()`, so `LLVM_PROFILE_FILE` flows
  test binary → `sgt` client → detached daemon.
- The daemon handles SIGTERM and exits by returning from `main`
  (`src/daemon.rs`); no `std::process::abort`, no `panic = "abort"` —
  profiles flush on the normal teardown path.
- The `DataDir` guard reaps SIGTERM-first with a 10 s grace before SIGKILL.

**Which suites spawn (corrected at S0, finding A1):** only m1 and m4 are
`sgt`-subprocess-free. m2 (8 tests), m3 (1), m5 (1), and m6 (7–8) all spawn
the binary and/or detached daemons — 17–18 of 218 test functions. m4's
children are `sh`/`git` stand-ins (uninstrumented, no loss).

## 6.1 Known loss sites (registered; §8 governs which are repaired)

1. `SpawnedDaemon::drop` (`tests/m6_surfaces.rs`) uses bare `child.kill()` —
   its two daemons (dashboard, doctor tests) flush nothing. Instrument
   repair, pinned per R-S0-5 (assert the child exited by SIGTERM, not
   SIGKILL).
2. `scripts/demo.sh` cleanup: bare SIGTERM, 5 s grace, **no escalation**,
   then `rm -rf` of the data dir — under instrumented shutdown this deletes
   the dir under a live daemon and the `DataDir` guard cannot see it
   (non-`DataDir` temp dir). Instrument repair: match the reaper's
   TERM→grace→KILL→verify-gone shape before `rm -rf`, fail closed if the
   daemon survives.
3. The `DataDir` SIGKILL fallback fires silently if instrumented shutdown
   exceeds 10 s — the reaper reports which signal was needed (repair,
   extends the existing reaper test).

## 6.2 Known measurement artifacts (not gaps; not repaired)

- The m6 static-scan tests (t5 family, SSE-vocabulary, stack pins) execute
  almost no product code by design.
- The two opt-in real-Claude tests are absent from the default run.
  `src/backend/claude.rs` (2,156 lines) is still substantially exercised by
  the 42 non-ignored m4 tests against a stand-in CLI — the loss is bounded
  to live-CLI-only regions. The baseline names those regions specifically;
  quantifying the live-path delta belongs to N2's real-Claude measurement
  (R-N0-6) and is handed there as an input, not held as an open-ended
  excuse.
- Pre-ruled test-only/no-production-caller regions that must NOT be filed as
  dead code (settled rulings; L3): the snapshot-load/`SnapshotBeyondJournal`
  path (backlog B1 — dormant by design) and `Analytics::table_rows` (M5
  ruling 3 — kept as the acceptance instrument).

## 6.3 Profraw integrity (R-S0-6)

The harness counts profraw files produced per stage, reports the count
merged vs. discarded, and fails the run if any profraw is rejected as
corrupt without being accounted for. A SIGKILLed process writes nothing
(under-report); a process killed mid-flush can write garbage (merge
failure) — both are counted, neither is silent.

# 7. Flake and timing discipline

Instrumentation slows execution ~1.5–3×. No suite is serialized today —
every suite runs the default parallel harness (the S0 challenge killed a
phantom `--test-threads=1` claim); the open question is whether
instrumentation's slowdown reintroduces the M3-era parallel-flake class.

**Flake census (R-S0-7):** two arms, both in default parallel mode —
**N=10 uninstrumented** full-suite runs (the control arm; matches the M3
"10 consecutive runs" precedent) and **N=3 instrumented** full-suite runs.
Census wall-time ceiling: ~4 h; if the ceiling would be exceeded, the census
is cut short and the actual N recorded (blocked-with-reason, never a silent
gap). A failure observed once that does not reproduce in 3 targeted re-runs
of that test is recorded by name as "observed, unreproduced" — it becomes an
issue only if it recurs across arms or runs. "Fails only under
instrumentation" may only be claimed with the control arm green.

**Timing exposure:** the at-risk population is *every wall-clock deadline in
`tests/`*, enumerated in the harness doc and ranked by measured headroom —
not by prominence. (The S0 challenge found the m5 rebuild bound carries ~27×
measured headroom while the unnamed 30 s daemon-boot descriptor deadline has
unknown headroom.) A timing test that fails only under instrumentation is a
finding for S2 with its own disposition — never silently loosened.

# 8. Fix policy (owner direction 2026-08-10, as ruled at S0)

Replaces P1-PERF's flat no-fix rule for this program. **No clause below
exempts anything from L7** (R-S0-5): every roll-in names its pinning shape
before it lands, and a repair whose pin would be unfalsifiable is moved to
"discuss first."

1. **Roll in** (S1 phase 1 only): instrument repairs and hygiene —
   §6.1's three repairs (pins: SIGTERM-exit assertion; demo.sh fail-closed
   verify-gone check, partial pin recorded honestly; extended reaper test),
   `.gitattributes` + ignore rules (§9), the CI trigger fix (§10, pinned by
   observation on the next push — config, not code), harness scripts and
   issue template. Rung-logged, separable commits (L10).
2. **Discuss first**: anything behavior-changing or breaking, anything
   touching `src/` beyond `#[cfg(test)]` modules, anything whose pin would
   be unfalsifiable, anything that would fire a B1–B3 trigger. Filed and
   flagged to the owner; not landed autonomously.
3. **Everything else**: filed as a labeled issue (§15.2) and tracked.

# 9. Repo hygiene

- `.gitattributes`: `linguist-vendored` on `reference/`, `reference-corpus/`,
  and `.sergeant/` (the latter two exist only on the N-branch today —
  marking them now means the language bar survives the N-merge unchanged);
  `linguist-generated` on `Cargo.lock`. Predicted result: ~91–92% Rust.
- `.gitignore`: add `__pycache__/` and `*.pyc` (both trees are already
  carrying tracked pycache artifacts; the rule prevents new ones).
- The already-tracked `reference/sergeant-upstream/.../__pycache__/*.pyc` is
  **left in place** (R-S0-8): `reference/` is committed evidence pinned to an
  upstream SHA by `reference/UPSTREAM.md`; untracking it would break the
  byte-identity claim for zero language-bar gain (the `.pyc` is binary and
  never counted). The N-branch's own pycache files are the N-series' to
  handle; noted for the merge.

# 10. CI policy (owner direction, 2026-08-10)

nim-proxy's measured mistake: every PR ran a 10–15 min full gate battery,
which taxed prototyping far more than it protected a two-person project.
Standing policy here:

- The per-PR job (fmt, clippy, test, warm cache) stays the only required
  check — but its trigger is fixed in S1 phase 1: today `on: [push,
  pull_request]` runs the battery **twice** per PR push; it becomes
  `push: branches: [main]` + `pull_request`.
- Coverage runs as a **separate, non-blocking lane**: `workflow_dispatch` +
  weekly `schedule`, never a required check, summary to
  `$GITHUB_STEP_SUMMARY`, no third-party report host, no new secrets.
- A `--fail-under-lines` gate appears only in that lane (S2 close-out),
  pinned *below* the measured number so noise never blocks work
  (nim-proxy's spread policy: gate 90 against measured 96).
- Revisit trigger: external contributors, or a release cadence.

# 11. Parallel-program protocol (N-series coexistence)

- File ownership is disjoint at the code level: S-series owns `src/`,
  `tests/`, `scripts/`, `docs/coverage/`, `docs/gauntlet/contracts/S*.md`,
  `docs/gauntlet/notes/s*`, `resources/`, `.github/`; the N-series owns
  `.sergeant/`, `reference-corpus/`, `docs/icm/`,
  `docs/gauntlet/{N*,notes/n*,runs}`. `Cargo.toml` is owned by *neither*
  program (R-S0-10): the S-series has no current need to touch it; a
  test-dependency need in S2 is contract material, not ambient authority.
- Shared-append files, resolved by whoever merges second, keeping both
  sides ordered by date: `GAUNTLET.md`, `LESSONS.md`, `CLAUDE.md`, **and
  `reference/UPSTREAM.md`** (the N-branch modifies it; S-series no longer
  does after R-S0-8). S-entries assume D9/N0/N1/L11 exist at merge time.
- Expected merge order: the S1 PR is small and code-adjacent and should
  merge before PR #27 closes N2; if the order flips, the S-side rebases —
  the `.gitattributes` already anticipates the N-trees.
- S-series workflow scripts are committed as plain `.js` under `resources/`
  (owner direction 2026-08-10). This supersedes the zip-append convention
  for the S-series; `reference/gauntlet-workflows.zip` remains the M/N
  archive.

# 12. Standing rulings inherited

- R-N0-6: issue #19 (real-Claude soak) is narrowed into N2's real-Claude
  measurement. S-series does not re-take it; the baseline hands N2 the named
  live-path-only regions (§6.2) as an input.
- B1/B2/B3 triggers are unchanged; coverage work that would fire one is a
  discussion item, not an autonomous change.

# 13. Non-goals

- No feature work, no perf fixes (#4–#13 keep their owners), no refactors
  beyond what a failing-first test demands.
- No JS coverage: `web/dashboard.js` is outside this instrument's reach
  entirely; #21 needs a different instrument and stays open, untouched.
- No mutation-testing program (a candidate later phase, only if S2's results
  argue for it; probes remain L5-bound to disposable copies).
- No coverage of `reference/` or the perf scripts; no branch-coverage
  promises; no per-PR coverage gate; no Codecov or external report host.
- No new API surface for testability — `tests/m6_surfaces.rs` t5 pins
  `ApiViews`, and SIGTERM already is the graceful stop path; a shutdown
  endpoint is R1-rejected.
- No `--ignored` runs of the real-Claude pair (token spend; R-N0-6).

# 14. Unknowns

- Actual instrumented wall-time and profraw volume (estimates: 40–60 min
  cold, 0.5–2 GB per full run; measure in S1 phase 2 and record).
- Whether `LLVM_PROFILE_FILE`'s pattern as set by cargo-llvm-cov is absolute
  (a relative one scatters profraws into deleted temp dirs — m2's helper
  sets the client cwd to a `TempDir`; verify on first run, first).
- Whether instrumentation's slowdown reintroduces the M3-era parallel-flake
  class in the default parallel harness (§7's census answers this).
- cargo-llvm-cov 0.8.7's actual `#[cfg(test)]`, exclusion, and
  failure-mode semantics (doctrine 1: measured by the builder, recorded in
  the harness doc).

# 15. Operating environment (added at S0)

1. **Gate regime (R-S0-1):** the no-mistakes pipeline is not present in this
   container (`no-mistakes` not on PATH, `/root/.no-mistakes` absent), so
   `scripts/gate.sh` cannot run. The S-series gate is the Bug Sprint 1
   precedent: orchestrator-verified `cargo fmt --check` + `clippy
   --all-targets -- -D warnings` + `cargo test`, plus the hygiene sweep
   (zero leaked daemons via the quoted pgrep rule — pattern extended to
   `llvm-cov-target/debug/sgt` — and zero `/tmp` residue). Recorded per
   milestone in the ledger. If the pipeline returns to the environment,
   the checkpoint-gate stage resumes with it.
2. **Issue filing (R-S0-11):** the orchestrator files issues through the
   GitHub MCP tooling this session holds (the `gh` CLI is absent). Fallback
   if filing fails at run time: findings land as
   `docs/coverage/findings-<date>.md` in the same commit as the baseline,
   transcribed to issues when tooling allows — never dropped.
   Granularity: deduped by subsystem and root cause (P1-PERF's #5
   precedent), capped at 12 issues, each naming the unexercised *behavior*,
   not bare line ranges. S1 delivers
   `.github/ISSUE_TEMPLATE/coverage-gap.yml` with `labels: ["coverage"]`,
   mirroring the perf template's evidence discipline.
3. **Disk budget (R-S0-6):** 29 GB free at S0. Hard rules: the harness
   pre-flights `df` and refuses to start any stage under 10 GB free;
   profraws are cleaned between census runs (`cargo llvm-cov clean
   --profraw-only` — flag verified by the builder); at most two build trees
   exist at any moment (`target/`, `target/llvm-cov-target`); nothing ever
   points at a shared or foreign cache.
4. **Session continuity:** the program runs autonomously in a remote
   container; anything worth keeping is committed and pushed promptly after
   its gate passes (the container is ephemeral).
