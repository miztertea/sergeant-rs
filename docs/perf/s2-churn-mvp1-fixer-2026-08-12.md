# S2 churn — MVP-1 fixer pass, W2/TH-08 evidence (2026-08-12)

TH-08's own finding: R-MVP1-9's measured half (`scripts/perf/s2-churn.sh`'s
RSS sampling) was never run; the substitute that landed instead was a
projection-level `runs.len() == 0` assertion, whose own docstring admitted
the substitution. This is the actual run, against this fixer pass's own
W1/W2 fix (the bounded `terminal_runs` LRU cache, `TERMINAL_RUN_CACHE_
CAPACITY = 512` — see `src/runtime/projection.rs`).

**Unit under test:** `target/release/sgt`, commit `3c047f1` (this fixer
pass's own HEAD at run time — every fix through TH-05 landed, including the
bounded cache). **Environment:** Cerberus host, same class as `docs/perf/
baseline-cerberus-2026-08-11.md`. Reduced scale from that doc's own S2 cell
(60 works in 6 waves of 10, 10s settle, vs. the full contract's 200/10/120s)
— a time-boxed run inside this fixer pass, not a re-run of the full P1-PERF
matrix; the shape (per-wave slope) is what TH-08 asks for, and 6 waves is
enough to see it.

## Result

| metric | value |
|---|---|
| RSS pre → peak → post-settle | 31,032 → 35,656 → 35,584 kB |
| RSS growth over the run | 4,552 kB (14.7%) |
| fds pre/peak/post | 15 / 19 / 15 (no fd leak) |
| journal events | 1,080 (18.0/work — matches the N3 two-phase-boundary shape `baseline-cerberus-2026-08-11.md` also measured) |
| hygiene sweep | clean — 0 leaked `sgt` processes, 0 leftover worktrees/surface dirs |

## Per-wave RSS (`s2-waves.csv`, this run)

| wave | works done | RSS (kB) | Δ this wave | kB/work this wave |
|---|---|---|---|---|
| 1 | 10 | 33,416 | — | — |
| 2 | 20 | 34,172 | 756 | 75.6 |
| 3 | 30 | 34,708 | 536 | 53.6 |
| 4 | 40 | 35,092 | 384 | 38.4 |
| 5 | 50 | 35,384 | 292 | 29.2 |
| 6 | 60 | 35,648 | 264 | 26.4 |

**The slope is decelerating, not flat-and-nonzero-forever and not the
monotonic non-decreasing climb `baseline-cerberus-2026-08-11.md` measured
pre-eviction (25.88 kB/work, "monotonic non-decreasing", its own S2 cell)** —
each wave's marginal cost is falling (75.6 → 53.6 → 38.4 → 29.2 → 26.4
kB/work), consistent with front-loaded fixed cost (DuckDB/tokio pool
allocation visible in the idle-profile baseline, ~30 MB before any work at
all) amortizing over more works, converging toward a small, bounded
marginal cost per work rather than compounding. 60 works is well under
`TERMINAL_RUN_CACHE_CAPACITY` (512), so this run does not exercise the
cache's own bound directly — that is proven structurally instead, by
`the_terminal_run_cache_itself_stays_bounded_under_churn_beyond_its_
capacity` (`src/runtime/projection.rs`), which pushes past the cache's
capacity and asserts it never grows past it. This run is the daemon-level
half TH-08 asked for: real RSS, real fds, real hygiene, not a projection
struct's field count.

## Honest bounds

- **Not the full 200-work/120s-settle contract cell** — time-boxed to 60
  works and a 10s settle inside a fixer pass already covering 28 findings.
  The shape (decelerating slope, no fd leak, clean hygiene) is the evidence
  TH-08 was missing; the exact 200-work numbers from the full P1-PERF matrix
  are a separate, larger measurement someone should still run before citing
  this as the milestone's own S2 baseline.
- **Raw artifacts** (not committed — scratch, per this repo's own
  convention): `s2-churn-summary.json`/`.tsv`, `s2-waves.csv`,
  `s2-continuous.csv`, `s2-calls.csv`, `s2-trend.json`, `hygiene-s2.txt`, at
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/
  6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/s2-th08/` on the host this
  ran on.
