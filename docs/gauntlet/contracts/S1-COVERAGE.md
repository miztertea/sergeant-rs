# S1 Contract — Coverage Baseline

Measurement milestone (P1-PERF template, amended by the S-series fix policy
§8). Governed by `reference/proposal-s-series-stabilization.md` (§4.1, §5,
§6, §7, §15) and the S0 rulings. This phase measures and repairs the
*instrument*; it fixes nothing in the product. Product findings become
issues.

## Outcome

A rerunnable coverage harness under `scripts/coverage/`, a dated baseline
`docs/coverage/baseline-2026-08-10.md` whose numbers a stranger can
reproduce from the committed command lines alone (R-S0-3), a known-losses
register, a two-arm flake census, and the confirmed findings filed per
R-S0-11 — plus the phase-1 hygiene set (`.gitattributes`, ignore rules, CI
trigger fix, issue template). Every deliverable through the R-S0-1 gate
regime.

## Instrument under test

- cargo-llvm-cov 0.8.7 (source-installed), rustup `llvm-tools`, rustc
  recorded per run via `rustc -vV` (R-S0-2 drift guard).
- Dev profile — the same profile the repo's own gates run; the
  `[profile.dev.package.libduckdb-sys] debug = false` pin applies. Recorded
  divergence from P1-PERF's release rule: coverage measures what
  `cargo test` exercises, not what ships.
- Collection target dir: cargo-llvm-cov's default `target/llvm-cov-target`;
  never `target/`, never a shared cache.
- Phase-2 baseline SHA: the tip of phase 1, quoted in the baseline doc.
- Container: 4 cores / 15 GiB RAM / disk per R-S0-6 pre-flight.

## Phase 1 — instrument & hygiene (commits, then freeze)

1. `SpawnedDaemon::drop` SIGTERM-first teardown; pin: a test asserting the
   dropped daemon exited by SIGTERM, not SIGKILL (R-S0-5).
2. Reaper reports TERM-vs-KILL per reaped pid; pin: extend the existing m2
   reaper test.
3. `scripts/demo.sh` teardown matches reaper shape (TERM → grace → KILL →
   verify-gone) and fails closed before `rm -rf`; partial pin recorded.
4. `.gitattributes` (`reference/`, `reference-corpus/`, `.sergeant/`
   vendored; `Cargo.lock` generated), `.gitignore` `__pycache__/`+`*.pyc`.
5. `.github/workflows/ci.yml` trigger: `push: branches: [main]` +
   `pull_request` (kills the double run).
6. `.github/ISSUE_TEMPLATE/coverage-gap.yml`, `labels: ["coverage"]`,
   uncovered-behavior-not-lines discipline.
7. The harness itself: `scripts/coverage/` stage scripts embedding the
   R-S0-3 command lines, disk pre-flight, version recording, profraw
   accounting (§6.3), hygiene sweep, and a harness doc enumerating every
   wall-clock deadline in `tests/` ranked by measured headroom (§7).

Gates green at phase close; separable commits (L10); rungs logged.

## Phase 2 — measurement matrix (strictly sequential, frozen tree)

| Stage | What runs | What is recorded |
|---|---|---|
| C0 | `cargo llvm-cov show-env`; verify `LLVM_PROFILE_FILE` pattern is absolute (§14's first Unknown) — hard stop if not | env output, verdict |
| C1 | `--no-report --lib` (88 unit tests) | wall time, profraw count, pass/fail |
| C2 | `--no-report --test m1_event_core`, then m4, m3, m5 | same, per suite |
| C3 | `--no-report --test m2_daemon_api`, then m6 (the spawning-heavy suites) | same + per-stage hygiene sweep + profraw delta proving daemon flushes arrived |
| C4 | `report --summary-only`, `--lcov`, `--html` | the baseline table, per-module |
| F1 | control arm: N=10 uninstrumented `cargo test` (R-S0-7) | failures by name |
| F2 | instrumented arm: N=3 full-suite via the C1–C3 pipeline | failures by name, wall-time ratio vs F1 |

Every stage: disk pre-flight (≥10 GB), post-stage
`pgrep -f "llvm-cov-target/debug/sgt --data-dir"` finds nothing, profraw
accounting balances. Census profraws cleaned between runs. Any instrument
defect found here → fix, commit, restart phase 2 (R-S0-4).

## Phase 3 — publication

Baseline doc with: convention command lines quoted verbatim; per-module
coverage table; known-losses register (§6.1–§6.2 verified against what
actually happened, including the named live-path-only `claude.rs` regions
handed to N2); flake census results both arms; every matrix cell filled or
blocked-with-reason. Findings deduped/capped/filed per R-S0-11 with URLs in
the ledger entry. GAUNTLET.md S1 entry (two scorecards, house shape).
LESSONS entry only if something general was learned.

## Findings discipline

A coverage finding = an unexercised *behavior* (not a line range) with the
module, the evidence (region list), a category (untested error path, dead
code candidate, race window, missing fixture, artifact-of-instrument), and
a suggested test shape. A flake finding = test name + arm + rate + repro
command. Pre-ruled regions (§6.2: B1 snapshot path, `table_rows`) are not
filable. #19-adjacent live-path regions are handed to N2, not filed.

## Non-goals

Product fixes of any kind; timing-bound changes (S2 findings); `--ignored`
real-Claude runs; JS coverage (#21); mutation testing; CI coverage lane
(S2 close-out); touching `Cargo.toml` (R-S0-10); any new API surface.

## Acceptance

1. A stranger with this container can rerun phase 2 end-to-end from
   `scripts/coverage/` alone and get comparable numbers (same convention,
   drift guard verifies).
2. Both baseline numbers and census results published with zero silent
   gaps; every known-loss site either repaired-and-pinned (§8.1) or
   registered with its expected effect on the numbers.
3. All phase-1 pins fail when their repair is reverted (revert-probed in a
   disposable copy, L5/L7; demo.sh's partial pin exercised to its recorded
   limitation).
4. Gate regime green at each phase close; hygiene sweep clean; ≤2 build
   trees at all times.
5. Findings filed (or fallback-committed) per R-S0-11; none dropped.

## Unknowns

Carried from proposal §14 (profraw pattern absoluteness — resolved at C0;
instrumented wall/space cost; parallel-flake reintroduction; cargo-llvm-cov
`#[cfg(test)]`/exclusion/failure-mode semantics — builder measures all
four and records them in the harness doc).

## Gauntlet depth

Role-based (P1-PERF precedent), one workflow committed at
`resources/s1-coverage-gauntlet.js`: one Opus-high builder for phase 1
(instrument repairs + harness, gates green, revert-probes its own pins);
phase 2 executed by the orchestrator running the committed stage scripts
sequentially with Sonnet analysis runners parsing each stage's artifacts;
one batched Opus refuter pass over candidate findings (reproduce-or-refute
against the lcov/html evidence and the tree; probe hygiene per L5); no
product-fix round exists in this phase by design — instrument-defect fixes
restart phase 2 per R-S0-4. Orchestrator adjudicates, writes the baseline
doc and ledger entry, and files findings.
