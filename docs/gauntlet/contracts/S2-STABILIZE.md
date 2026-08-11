# S2 Contract — Stabilization

Test-only remediation milestone (M-template, narrow). Governed by the
S-series proposal (§4, §8, §10) and rulings R-S0-1..13. In scope: **issues
#30–#41, cited by number, never restated** — plus the narrowed residuals
preserved in the S1 analysis record (workflow `wf_10dbc0d3-42d`) where a
wave's builder finds them adjacent to in-scope work. Baseline of record:
`docs/coverage/baseline-2026-08-10.md` (91.43% lines at `dc77de9`, with
the `#[cfg(test)]`-inflation caveat).

## Outcome

Every in-scope issue closed by a test-only diff or closed-with-reason by
adjudication; every new test falsifiable (fails on the pre-fix/reverted
tree — L7, revert-probed or mutation-probed per L5); coverage re-measured
by the committed S1 convention; the non-blocking CI coverage lane lands as
the close-out commit (R-S0-9) with `--fail-under-lines` set below the
re-measured number (nim-proxy spread, proposal §10). Residual gaps written
down, never chased silently (doctrine 3).

## Hard rules

- **Test-only**: diffs touch `tests/`, `#[cfg(test)]` modules in `src/`,
  test fixtures, and `scripts/coverage/` (instrument refinements only).
  Zero production-behavior change. A gap unreachable without a production
  seam is a *finding for adjudication* (YAGNI-first, doctrine 2), never an
  autonomous seam.
- `Cargo.toml` untouched (R-S0-10); a test-dep need escalates to the
  owner (§8.2).
- Pre-ruled regions stay unfiled and untested-by-decree (§6.2: B1
  snapshot path, `table_rows`); real-Claude live paths stay N2's
  (R-N0-6); no `--ignored` runs.
- Tests must hold under the census discipline: any new test that flakes in
  a 10-run check is not done.

## Waves

- **W1 — fixtures and in-module pokes** (no fault injection): #32 domain
  validation, #39 fake-backend grammar, #41 client fixtures, #33
  projection guards, #34 telemetry fold, plus #30's fixture-shaped items
  (empty trailing segment, malformed first line) and #37's pure decoder
  cases.
- **W2 — fault injection and integration** (io/error/race): #30's
  poison/rollback pair, #31 storage helpers, #35 daemon
  committer/export, #36 api idempotency, #37's SSE race pair, #38 cli
  fallbacks/doctor, #40 surface fallbacks.
- **W3 — close-out**: blind-auditor sweep over what W1/W2 left uncovered
  (the nim-proxy round-2 pattern: cheap tricks the earlier waves missed),
  re-measure via `scripts/coverage/` C0–C4 + a 10-run flake check on the
  final tree, then the CI lane commit
  (`.github/workflows/coverage.yml`: `workflow_dispatch` + weekly
  schedule, `$GITHUB_STEP_SUMMARY`, never required) proven by one
  dispatch run.

Waves are sequential; within a wave, builders run on disjoint file
surfaces (in-module `src/` test mods vs. `tests/` suites) or in isolated
worktrees merged by the orchestrator.

## Acceptance

1. Each issue's closing commit names it (`Fixes #NN` or
   closed-with-reason in the ledger), one issue-cluster per commit (L10).
2. Every new test revert-probed (or mutation-probed where the "fix" is
   the test itself: the probe deletes the behavior's guard in a
   disposable copy and the new test fails).
3. Re-measured coverage ≥ baseline on lines and functions, quoted with
   the same caveat; no silent convention change (R-S0-3).
4. Census: 10/10 uninstrumented + 3/3 instrumented green on the final
   tree; gate regime (R-S0-1) green at every wave close.
5. CI lane merged, one green `workflow_dispatch` run recorded, gate set
   per §10 and quoted in the ledger entry.

## Non-goals

Production fixes of any kind (issues found in product behavior while
writing tests → filed, flagged §8.2, never fixed here); perf issues
#4–#13; JS coverage (#21); mutation-testing program; touching the
N-series' surfaces.

## Unknowns

- Whether every #30/#31 io-error arm is reachable without a seam (the S1
  suggested shapes say yes — dir-as-file, read-only handles, in-module
  field pokes; W2's builder verifies or escalates).
- Whether the telemetry fold (#34) is testable against the existing
  `opentelemetry_sdk` testing feature without new dev-deps (R-S0-10
  pressure; escalate if not).
- The re-measured number W3 gates against.

## Gauntlet depth (R-S0-13 seats)

Wave builders: **Sonnet, medium-high** — every task has an enumerated
contract (the issue), grounded inputs (lcov evidence), and a checkable
output (a test that fails on revert). Panel after each wave: lean
two-critic round — **test-honesty on Opus-high** (independently re-probe a
sample of every builder's pins; any self-probe-only pin is a finding) and
**invariants/simplicity on Opus-medium** (no production drift hiding in a
"test-only" diff; no fixture machinery beyond need) — batched Opus
refuters, Sonnet fixer for enumerated corrections, Opus fixer if a fix
needs architectural judgment. W3's blind auditor: **Opus-high** (judgment
under breadth over the residual map). Orchestrator: adjudication, wave
merges, re-measurement runs, ledger, CI-lane commit.
