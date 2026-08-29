# S1 coverage harness

The instrument behind `docs/coverage/baseline-2026-08-10.md`. Everything here
collects, merges and reports coverage for `src/**`; nothing here changes the
product, and no stage is allowed to produce a number by a route other than the
committed command lines (R-S0-3 — *a number produced any other way is not this
program's number*).

```sh
scripts/coverage/c0-show-env.sh            # environment + profraw-pattern verdict (hard stop)
scripts/coverage/c1-lib.sh                 # 88 unit tests, --no-report
scripts/coverage/c2-suites.sh              # m1, m4, m3, m5 — one sub-stage each
scripts/coverage/c3-spawning-suites.sh     # m2, m6 — the suites that drive real processes
scripts/coverage/c4-report.sh              # one merge, three exports
scripts/coverage/f1-control-census.sh 10   # flake census, control arm (uninstrumented)
scripts/coverage/f2-instrumented-census.sh 3  # flake census, instrumented arm — AFTER C4
```

Strictly in that order, one at a time, against a frozen tree (R-S0-4). There
is deliberately no `run-all.sh`: phase 2 is driven stage by stage so that a
stage's artifacts are read before the next one starts.

## The convention, stated once

| constant | value | why |
| --- | --- | --- |
| profile | `dev` | the profile the repo's own gates run; coverage measures what `cargo test` exercises, not what ships (diverges from P1-PERF's release rule, deliberately) |
| collection tree | `target/llvm-cov-target/` | cargo-llvm-cov's own; never `target/`, never a shared cache (docs/DEVELOPMENT.md's thrice-bitten rule). Two build trees exist at most, which is R-S0-6's ceiling |
| report scope | `src/**` | the tool's default exclusions already achieve it — measured below, no flag needed |
| `--ignored` | never run | the two opt-in real-Claude tests spend tokens; their absence is a registered known loss, not an oversight |
| toolchain | recorded per run | `rustc -vV` + `cargo llvm-cov --version` into the artifacts dir; a mid-run change is a hard stop (R-S0-2) |
| disk floor | 10 GB free | pre-flighted per stage; a truncated profraw takes the whole report down (measured below) |

Artifacts land in `docs/coverage/artifacts-2026-08-10/` (override with
`COV_ARTIFACTS`). They are **committed** — they are the evidence the baseline
is read from — so they hold logs, counts and verdicts only. Profraws, profdata
and the HTML tree stay out of git: the bulky exports go to
`target/llvm-cov-reports/` (`COV_REPORTS`), and the artifacts dir records their
sizes and the lcov's SHA-256 instead.

### Where the stage scripts differ from §5's quoted block, and why

The seven collection lines and `report --summary-only` appear verbatim. Two
report lines carry an argument the proposal wrote as a placeholder or left to
the default, both recorded here so the baseline can quote the real thing:

- `report --lcov --output-path "$COV_REPORTS/lcov.info"` — §5 writes
  `<artifacts>/lcov.info`. `<artifacts>` resolves to `target/llvm-cov-reports/`
  rather than the committed evidence dir, because the lcov is megabytes of
  regenerable derivative and the committed dir is meant to stay readable.
  Its size and SHA-256 are committed instead.
- `report --html --output-dir "$COV_REPORTS/html"` — §5 writes bare
  `--html`, whose default output is `target/llvm-cov/html`. The explicit
  directory keeps every export in one place beside the lcov, and keeps a
  third `target/` subtree from appearing next to the two build trees R-S0-6
  allows.

Other knobs: `COV_MIN_FREE_GB` (default 10), `COV_CENSUS_CEILING_S` (default
14400 — R-S0-7's ~4 h ceiling; a census that would cross it stops and records
the shortfall by name), `COV_ALLOW_DRIFT=1` (accept a toolchain change as a
deliberate re-baseline).

## What every stage does

- **Disk pre-flight** — `df` against the repo root, refuse under the floor.
- **Toolchain fingerprint** — write `toolchain.txt` on first use of an
  artifacts dir; on every later stage, compare and refuse on a mismatch.
  Profdata from two rustc versions must never be merged into one number.
- **Profraw accounting** — the profraw set is listed before and after; each
  newly produced file is individually run through the toolchain's
  `llvm-profdata merge` to classify it *mergeable* or *discarded*. Counts land
  in `<stage>-accounting.tsv`; anything unmergeable is named in
  `<stage>-profraw-discarded.txt` and fails the stage, because one such file
  is enough to kill the whole report (measured below). A stage that produces
  fewer profraws than its floor fails too — the spawning suites' floor is what
  proves subprocess flushes actually arrived. The accounting runs in **both**
  directions: files that were present when the stage began and are gone at the
  end are counted as `profraw_lost`, named in `<stage>-profraw-lost.txt`, and
  fail the stage. Additions-only accounting could not fail on loss at all, and
  loss is the failure mode the staged shape is exposed to — the entire
  C1–C3 → C4 pool rests on measured claim 3 below, and if a tool bump ever
  falsified it, every stage would still clear its floor and C4's guards
  (profraws > 0, profdata mtime moved) would still pass while the baseline
  quietly described the last suite only. One stage loses profraws on purpose:
  F2's census passes open with `clean --profraw-only`, so F2 declares it with
  `cov_expect_profraw_loss`, which records the loss and its reason in the tsv
  instead of failing. That declaration is the *account* in "unaccounted loss"
  (R-S0-6) — there is exactly one, and it is in the committed evidence.
- **Hygiene sweep** — `pgrep -f "llvm-cov-target/debug/sgt --data-dir"` and the
  uninstrumented equivalent must both find nothing; `/tmp` residue is counted
  and recorded. Quoting matters: an unquoted pattern matches the shell that
  typed it. Inside a script file it cannot (pgrep skips itself, and the
  parent's argv is the script path) — the same line pasted into `bash -c`
  finds itself, which is how this check has been fooled before.

  The two halves are deliberately not enforced alike, which R-S0-1's gate
  regime ("zero leaked daemons, zero `/tmp` residue") does not itself
  distinguish: **a leaked daemon fails the stage; `/tmp` residue is recorded
  and does not**. A leaked daemon is unambiguous — it holds a deleted data dir
  and its profile never arrived. `/tmp` is shared with everything else on the
  container, and this repo's own suite legitimately has a `sgt-demo-*`
  directory in flight while m6's `t4` runs, so a count taken at a stage
  boundary cannot tell that from residue. The number is therefore evidence a
  reader compares across stages (a count that only ever grows is the signal),
  not a gate. Recorded here rather than left to be inferred from the code.

### What C0 does and does not prove

C0 resolves proposal §14's first Unknown by parsing `LLVM_PROFILE_FILE` out of
`cargo llvm-cov show-env` and hard-stopping unless it is absolute and carries
`%p`. Worth being exact about the scope, because measured claim 1 below says
the two flows differ: **the string C0 checks belongs to the `show-env` flow,
and every collection this harness runs is a *managed* run** (`cargo llvm-cov
--no-report …`), which writes elsewhere. Both patterns were measured absolute
on 0.8.7, so C0's verdict is currently true of both — but a future tool bump
that made only the *managed* pattern relative would not trip C0. What would
catch that is C1–C3's profraw floor: a relative pattern scatters profiles into
the `TempDir` cwds the suites delete, and the stage's produced-count collapses
below its floor. C0 is the cheap early stop, not the defense.

## Measured behavior of cargo-llvm-cov 0.8.7

Doctrine 1 / L1 at the tool boundary: none of the following is quoted from
documentation. Each was produced on this container (rustc 1.94.1, LLVM 21.1.8,
cargo-llvm-cov 0.8.7) against a throwaway crate outside this tree (L5), on
2026-08-10. The transcript — commands and their real output, one linear pass —
is committed at
`docs/coverage/artifacts-2026-08-10/tool-probes-2026-08-10.txt`; the P-numbers
below are its section markers. Rerun cost: seconds.

1. **The collection directory is not what `show-env` says.** `show-env` prints
   `CARGO_LLVM_COV_TARGET_DIR=<root>/target` and
   `LLVM_PROFILE_FILE=<root>/target/<crate>-%p-%4m.profraw`. A *managed* run
   (`cargo llvm-cov …`, which is what this harness uses) instead builds into
   and writes profraws under `<root>/target/llvm-cov-target/`. Both paths are
   absolute; they are different directories. C0 records the show-env value and
   names the managed one, and C1–C3 count profraws in the managed one.
2. **The profraw pattern is absolute and per-process** — `%p` (pid) and `%4m`
   (a four-way merge pool). This resolves proposal §14's first Unknown: a
   relative pattern would have scattered subprocess profiles into the
   `TempDir` cwds that m2's helper deletes. C0 hard-stops if this ever changes.
3. **`--no-report` pools; it does not clean.** Three successive `--no-report`
   stages left 1, then 2, then 3 profraws. The staged C1–C3 → C4 shape depends
   on this and would silently report only the last suite if it were false.
4. **Report scope is `src/**` by default; no flag is needed.** The tool passes
   its own `-ignore-filename-regex` to `llvm-cov`, covering
   `/rustc/<hash>/`, `<root>(/.*)?/(tests|examples|benches)/`,
   `<root>(/.*)?/(tests\.rs|[0-9a-zA-Z_-]+[_-]tests\.rs)$`,
   `<root>/target/llvm-cov-target`, `~/.cargo/(registry|git)/` and
   `~/.rustup/toolchains`. Consequences here: `tests/*.rs` and
   `tests/support/mod.rs` are excluded from the *report* while still being
   instrumented and still contributing coverage of `src/**`; and any future
   `src/**/foo_tests.rs` would vanish from the report without anyone asking.
   (The per-function array of a full `--json` export still lists the `tests/`
   functions; the per-file summary, the lcov and the tables do not.)
5. **`#[cfg(test)]` code inside `src/` IS counted, as covered.** In the probe,
   a `lib.rs` with four public functions and three functions inside
   `#[cfg(test)] mod tests` reported seven functions, with all three test
   functions at execution count 1. There is no stable-Rust way to exclude them
   without editing `src/` (`#[coverage(off)]` is nightly, and R-S0-10/§8 put
   `src/` out of this program's reach), so this is a **registered measurement
   artifact, not a defect**: 3,396 of `src/`'s 18,861 lines — 18.0% — live
   below a `#[cfg(test)]` marker, concentrated in `tui.rs` (41%),
   `surface.rs` (39%), `recovery.rs` (47%), `router.rs` (41%) and
   `fsutil.rs` (49%). Every headline `src/**` number is inflated by that
   fraction of near-fully-covered test code, and the baseline says so with
   the per-module table beside it.
6. **One corrupt profraw kills the whole report.** With a truncated file in
   the set, the default `--failure-mode any` makes `llvm-profdata merge` fail
   outright: `error: no profile can be merged`, `cargo llvm-cov report` exits
   1, no report at all. `--failure-mode all` instead warns, drops that file
   and reports from the rest (exit 0). The convention keeps the default — a
   loud failure is the honest outcome — which is exactly why each stage
   validates its own new profraws one at a time and names any casualty.
7. **A report with no profraws present is silently stale.** After
   `clean --profraw-only`, `cargo llvm-cov report` printed the *previous*
   run's numbers, exit 0, no warning: with nothing to merge it skips the merge
   step entirely and runs `llvm-cov report` against the leftover `.profdata`
   (mtime unchanged, verified). Nothing about the output says so. C4 therefore
   refuses to run with zero profraws and proves the merge happened by watching
   the profdata's mtime change.
8. **`clean --profraw-only` is cheap and narrow.** It removed the `.profraw`
   files and left the `.profdata`, the `*-profraw-list` and the entire build
   tree intact — so a census pass costs no rebuild. Note the corollary of
   points 7 and 8 together: cleaning profraws does *not* invalidate the stale
   report path, it creates it.
9. **Subprocess coverage propagates.** A test that spawned
   `env!("CARGO_BIN_EXE_…")` produced two profraws (test binary + child) and
   moved the child's `main.rs` from 0% to 100%. The mechanism this repo needs
   works at the tool level; whether it works through a *detached daemon* is
   what C3's profraw floor measures.
10. **Signals decide whether a profile exists at all** — the measurement
    behind all three of phase 1's repairs. An instrumented process was killed
    three ways: SIGTERM with the default disposition → **0 profraws**; SIGTERM
    with a handler that returns/exits → **1 profraw**; SIGKILL → **0
    profraws**. The at-exit writer runs only on a normal return from `main`.
    `sgt daemon` installs a tokio SIGTERM handler and returns from `main`
    (`run_until_signal`), so a SIGTERM-first teardown is precisely what makes
    its profile exist — and any SIGKILL in a run is a hole in the numbers,
    which is why both reapers now report which signal they needed.

Inherited, *not* re-measured here: that the system LLVM cannot read
rustc-21-era profraw (proposal §5). The harness sidesteps the question by
using the rustup component's tools exclusively —
`$(dirname $(rustc --print target-libdir))/bin/llvm-profdata`.

## Every wall-clock deadline in `tests/`

Proposal §7's at-risk population, enumerated rather than sampled: every
`Duration::from_secs` / `Duration::from_millis` in `tests/` and
`tests/support/`. Instrumentation costs an estimated 1.5–3×, so the ranking
that matters is *headroom*, not prominence — and headroom that has never been
measured ranks above headroom that has.

Two risk classes, and the second is the dangerous one:

- **Deadlines** bound a *polling* wait. Slowdown eats headroom; the test still
  passes as long as the operation finishes inside the bound.
- **Fixed sleeps** are a wait with no retry: the test sleeps, then asserts.
  Slowdown does not eat headroom here, it invalidates the premise. These are
  listed first.

### Fixed sleeps (no retry — a slowdown breaks the premise, not the margin)

| site | sleep | premise it assumes | headroom |
| --- | --- | --- | --- |
| `m6_surfaces.rs:502` | 1 s | the TUI's SIGHUP watch is installed by now; the pty is then hung up | **unknown** — if install slips past 1 s under load the hangup lands before the watch and the test fails for the wrong reason |
| `m4_backends.rs:1708` | 750 ms | after `stop` returned, *no further write* lands in the data dir | **unknown**, and asymmetric: a slower reader thread makes a late write more likely, i.e. instrumentation can turn this green→red honestly |
| `m5_projections.rs:1505` | 750 ms | a regression's force-flush would have reached the stand-in collector by now (asserts zero hits) | **unknown**, same asymmetry — the assertion is a negative, so slowness weakens it silently in the other direction |
| `m2_daemon_api.rs:1652` | 300 ms | client A is far enough ahead of client B to make the stale-descriptor race deterministic | **unknown** — both clients slow down together, but the 300 ms separation is absolute |
| `m4_backends.rs:3808` | 6 s | the real-Claude turn is genuinely in flight (measured ~3.5 s to first API activity) | ~1.7×, and **never run** — `#[ignore]`d, opt-in only |
| `m4_backends.rs:696`, `m5:1374`, `m6:498/513`, `support:251`, … | 10–500 ms | poll intervals inside deadline loops — not premises, just granularity | n/a |

### Deadlines, worst headroom first

| site | bound | what it bounds | measured | headroom |
| --- | --- | --- | --- | --- |
| `m2_daemon_api.rs:57` | 10 s | *every* HTTP request m2 makes (shared client timeout) | unknown | **unknown** — the tightest of the four client timeouts and the one covering the most calls |
| `support/mod.rs:31` + `m6_surfaces.rs:2425` | 10 s | daemon shutdown after SIGTERM, before SIGKILL | unknown | **unknown, and the one that costs coverage**: exceeding it means SIGKILL, which means that daemon's profile never exists. Both reapers now report the escalation instead of hiding it |
| `m4_backends.rs:1164…3077` (16 sites) | 10 s | a stub-backed turn settling (`wait_settled`) | unknown | **unknown** — the most-repeated bound in the suite; the stubs are shell scripts (uninstrumented) but the adapter around them is not |
| `m2_daemon_api.rs:1040/1042` | 10 s each | N SSE events to arrive; one chunk to arrive | unknown | unknown |
| `m4_backends.rs:405`, `m4:608` | 10 s | N normalized events / N recorded launches to appear (async reader thread) | unknown | unknown |
| `m4_backends.rs:2912` | 10 s | liveness settling to `Exited` after an orphan is killed | unknown | unknown |
| `m3_execution.rs:2260`, `m2:1254` | 10 s | a killed daemon's descriptor to disappear | ~0.8 s daemon boot as a proxy; shutdown unmeasured | unknown |
| `m2_daemon_api.rs:1188` | 15 s | `handle.shutdown()` with an SSE client attached | unknown | unknown |
| `m4_backends.rs:955` | 15 s | adapter events reaching the journal, read back through the API | unknown | unknown |
| `m5_projections.rs:1362` | 20 s | N telemetry spans exported | unknown | unknown — bounded by the exporter's batch interval, not by CPU |
| `m6_surfaces.rs:306` (+`:309`, 5 s per event) | 20 s / 5 s | an SSE-driven TUI refresh; each single event | unknown | unknown |
| `m5_projections.rs:2232` (+`:2237`, 2 s read) | 20 s / 2 s | the stand-in OTLP collector's accept loop | unknown | unknown |
| `m3:71`, `m5:66`, `m6:86` | 20 s | every HTTP request those suites make (shared client timeout) | unknown | unknown |
| `m2_daemon_api.rs:1206` | 5 s | the SSE stream closing after shutdown | unknown | unknown |
| `m6_surfaces.rs:1365` | 5 s | a `true` child's pid leaving the process table | ~ms | very large — kernel-bound, not load-bound |
| `m4:530`, `m5:1816`, `m6:2477` | 10 s | the ETXTBSY window on a just-written stub script | ~ms | very large — filesystem-bound |
| `m5_projections.rs:2178`, `m5:2197` | 30 s | the journal going quiet / all expected kinds journaled | unknown | unknown, but 30 s over a suite that finishes in 9.5 s total |
| `m4_backends.rs:146`, `m4:2959` | 30 s | a stand-in child reaching `exec` (argv visible in `/proc`) | ~ms | very large |
| `m6_surfaces.rs:511` | 30 s | the TUI exiting after its terminal hangs up | unknown | unknown |
| `m6_surfaces.rs:2341` | 30 s | a spawned daemon publishing its descriptor | **0.74–1.85 s** (5 samples, idle box, uninstrumented) | **16–40×** — the S0 challenge's "unknown headroom" deadline, now measured |
| `m5_projections.rs:2344` | 30 s | *asserted* rebuild time for 16,000 events — a performance bound, not a wait | **1.17 s** (13.7k events/s) | **25.6×** — the most generous bound in the suite, as S0 estimated (~27×) |
| `m6_surfaces.rs:486` | 90 s | the TUI coming up under a pty and painting | unknown | unknown, but the largest bound in the default suite |
| `w1b_overlay_lifecycle_trigger.rs:53` (`LIFECYCLE_DEADLINE`, 3 sites) | 120 s | S5 W1b's Work-overlay lifecycle hook: the overlay generation appearing after a surface binds, disappearing after it retires, and the surface binding at all — each a **polling** wait (50 ms `POLL_GAP`, never a sleep-then-assert) | **4.8 s / 4.2 s for the WHOLE test** including daemon boot, an estate scan, a Work submit, a cancel and a shutdown (2 samples, this container, uninstrumented — **not yet measured on the slowest supported target**, x86_64 Linux / Apple Silicon macOS / WSL2, per `docs/reference/glossary-and-support.md:18`) | **≥ 25×** against a conservative upper bound on this container — the individual waits are a fraction of those totals, and the bound is a **polling** wait (this section's own risk-class split, above), so a slower target eats headroom rather than invalidating the premise; the ≥25× margin is offered as reassurance, not as the cross-platform measurement the letter of the rule asks for, which is still open |
| `m4:3583/3593/3688/3793/3837`, `m4:3817` | 180 s / 30 s | real-Claude turns settling | n/a | **never run** — `#[ignore]`d, and R-S0-3 forbids `--ignored` here |

Suite wall times on this container, uninstrumented, for scale: m1 1.1 s,
m4 3.1 s, m2 6.6 s, m3 6.4 s, m6 8.8 s, m5 9.5 s, `--lib` 1.5 s. At the
estimated 1.5–3× slowdown, no *total* approaches any bound above — the risk is
concentrated in the individual operations whose headroom is marked unknown,
and in the four fixed sleeps.

One empirical check on all of it, run in phase 1 and recorded in
`docs/coverage/artifacts-2026-08-10/phase1-measurements.txt`: a full
`cargo test` takes 37 s idle, 71 s under four CPU hogs (~1.9×) and 92 s under
eight (~2.5×), **green every time**. That is a proxy for instrumentation, not
a substitute — contention slows everything uniformly, whereas instrumentation
slows instrumented code specifically and adds a profile write at exit, which
is a cost the fixed sleeps and the SIGTERM graces feel and a CPU hog does not.
But it does mean the unknown-headroom rows above are unknown, not suspected:
nothing in the suite is known to sit near its bound. F2 answers it properly. A
deadline that fails only under instrumentation is an S2 finding with its own
disposition, **never** a bound that gets quietly loosened.
