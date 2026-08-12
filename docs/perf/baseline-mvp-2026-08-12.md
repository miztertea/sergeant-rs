# Performance Baseline — MVP re-baseline, Cerberus (2026-08-12)

MVP-4's perf re-baseline of the assembled product (MVP-1 core, MVP-2
adapters/Docker, MVP-3 CLI all landed), same [P1-PERF contract](../gauntlet/contracts/P1-PERF.md)
harness (`scripts/perf/`) used for `docs/perf/baseline-cerberus-2026-08-11.md`
and `docs/perf/baseline-2026-08-10.md`. Every number below comes from the raw
run artifacts (`run-all-status.tsv`: all 8 stages `ok`) — nothing is
estimated; unsourced cells are marked "—". Token-free: fake backend
throughout, zero real-Claude spend (the real soak is a separate owner-gated
step, not run here).

**Unit under test:** `target/release/sgt`, commit `aedc7cb075d573854a7986a0b6d9078c4cf730e7`
("MVP-4 stabilize workflow (token-free half; real soak owner-gated)" —
every scenario's own `commit` field agrees, `commit_dirty_tree=0`,
`binary_swapped_since_pin=0`, `binary_predates_head=0` throughout: unlike
the 2026-08-11 run, no concurrent-commit drift this time — issue #50's fix
holds). **Environment:** Cerberus host, 20 cores (bare metal, no cgroup
quota), 32,058,072 kB RAM, `Linux 7.0.0-29-generic`, loopback HTTP, fake
backend throughout. `cargo build --release` immediately before the run
(12.19 s incremental; DuckDB unchanged).

## Idle profile

| metric | value |
|---|---|
| RSS (119×1s samples, 120s settle) | 32,508 → 32,540 kB, drift +32 kB (flat) |
| fds / threads | 15 / 42 |
| CPU over settle window | 0.12 s (0.1%) |
| cold start, empty data dir | 300.84 ms (single sample) |

Same shape as the 2026-08-11 baseline (29.9→29.8 MB there vs 32.5 MB here —
directional noise, not a regression; idle disk footprint now includes the
DuckDB file created at first start, 536,576 B in both runs).

## S1 — burst submissions (concurrent `POST /v1/work`, two-stage workflow)

| burst | wall | throughput | p50 | p95 | max | RSS peak | fds peak | events/work |
|---|---|---|---|---|---|---|---|---|
| 1 | 0.057 s | 17.7/s | 25.7 ms | — (1 sample) | 25.7 ms | 32.0 MB | 15 | 18.0 |
| 5 | 0.112 s | 44.7/s | 74.0 ms | 84.7 ms | 85.9 ms | 33.6 MB | 15 | 18.0 |
| 10 | 0.254 s | 39.4/s | 201.4 ms | 223.9 ms | 225.9 ms | 35.0 MB | 27 | 18.0 |
| 20 | 0.488 s | 41.0/s | 400.6 ms | 448.1 ms | 453.1 ms | 37.3 MB | 40 | 18.0 |
| 50 | 1.209 s | 41.4/s | 1000.4 ms | 1100.2 ms | 1109.1 ms | 44.6 MB | 67 | 18.0 |

All 50 works confirmed completed at every cell; 18.0 events/work uniformly
(unchanged N3 two-phase-boundary shape — the new manifest/envelope/Docker
code adds fields to existing events, not new event kinds, on this
workflow). Single-submit p50 (25.7 ms, single sample) beats the 2026-08-11
figure (41.3 ms) and holds the ≤50 ms R-N0-4 budget with room. Burst-50
throughput (41.4/s) is close to 2026-08-11's 42.0/s — within run-to-run
noise on the same host, no regression from the assembled MVP-1..3 code.

## S2 — sustained churn, 200 works in waves of 10, 120 s settle

| metric | value |
|---|---|
| submit phase wall (20 waves) | 5.131 s |
| RSS pre → peak → post-settle | 32,148 → 40,932 → **40,860 kB (zero reclaim)** |
| RSS per completed work (regression slope) | **43.56 kB/work, monotonic non-decreasing** |
| fds pre/peak/post | 15 / 27 / 15 |
| daemon CPU per work | 3.4 ms |
| events per work | 18.0 |

The no-reclaim shape reproduces (same retained-projection cause as both
prior baselines). **Directional finding, not a budget breach:** RSS/work
(43.56 kB) is ~1.7× the 2026-08-11 figure (25.88 kB/work) and ~1.7-2×
the container's ~21 kB/work. R-N0-4 sets no fixed €/work ceiling here — the
contract's bar is "returns near the pre-run mark" (it does not: consistent
across all three baselines) and "a monotonic climb is a finding" (recorded,
not new). The larger per-work retained footprint is consistent with MVP-1's
turn envelope + manifest/estate bindings now living on the in-memory Work
projection (more fields per work than the P1/N3-era shape); no journal
event-count growth accompanies it (still 18.0 events/work), so this reads as
increased in-memory Work-struct size, not new journal volume. Worth a
follow-up measurement if S2's full 200-work RSS slope becomes a tracked
budget; not chased further here.

## S3 — deep work (turn envelope hit at 12 turns) + graph read load

| metric | value |
|---|---|
| final state / stop reason | `blocked` — journal: `"turn envelope exhausted (12 turns)"` |
| respond cycles / journal events reached | 12 cycles / 81 events (target 1,000 — **not met, and correctly so**) |
| graph response at depth | 3,039 B (9 nodes / 8 edges) |
| respond cycles (12×) | p50 20.2 ms, p95 22.8 ms |
| sequential 100× graph reads | p50 13.7 ms, p95 15.6 ms |
| 20-way concurrent ×5 rounds | p50 32.5 ms, p95 38.9 ms (~2.4× amplification) |
| round wall (20 reqs) | 0.062–0.064 s across 5 rounds |
| RSS before/peak/after | 33.5 → 88.3 → 54.8 MB |
| fds | 15 / 27 / 15 |

**This is a real behavioral shift from the 2026-08-11 baseline (which reached
1,011 events on the same scenario), and it is by design, not a regression.**
MVP-1 landed the turn-count envelope at every turn-producing `Backend` verb
(the mvp-bucketing plan's MVP-1 row); this scenario's default workflow now
carries `turn_cap:12`, and the daemon correctly fails the work closed to
`blocked` with a named reason the moment the cap is hit, rather than letting
`work.respond` continue indefinitely — exactly the fail-closed behavior the
envelope was built to guarantee (confirmed from the journal:
`work.blocked {"reason":"turn envelope exhausted (12 turns)"}` at seq 84).
The read-path load (graph/work-show/events-tail) still ran at the depth
reached (81 events / 9 nodes) and shows the same latency shape as the prior
baselines at their much deeper mark — the read paths were never the bound.
Per P1-PERF's own rule ("a scenario that cannot hit its target ... records
where it stopped and why — that is data, not failure"), this is recorded as
data: the S3 scenario's `PERF_S3_TARGET_EVENTS=1000` knob is now
incompatible with the shipped default turn cap and needs raising
`PERF_S3_MAX_CYCLES`'s effective ceiling (a workflow/profile override) for a
future run that wants to re-exercise the harness at depth — a harness note,
not a product defect.

## S4 — SSE fan-out: 25 subscribers through a 20-work burst

Exactly 1 fd per subscriber (15 → 40, unchanged). Phase 1: burst wall
0.501 s, submit p50 408.9 ms / p95 448.6 ms, 25/25 subscribers saw all 20
terminal events. Phase 2: 10 subscribers killed mid-flight, burst wall
0.481 s, 20/20 HTTP 201, 15/15 survivors saw all events, fds returned exactly
to 15 (0 leaked), a post-kill submit completed normally (`post_kill_wedged:
no`). Matches both prior baselines' shape exactly — no regression.

## S5 — journal scale: cold-start rebuild + analytics

| mark | cold start | rebuild rate (raw) | RSS after start | journal size |
|---|---|---|---|---|
| 10k events | 441.37 ms | 23,461 ev/s | 74.7 MB | 6.6 MB |
| 25k events | 643.13 ms | 39,199 ev/s | 111.4 MB | 16.0 MB |
| 50k events | 1028.24 ms | 49,030 ev/s | 161.2 MB | 32.0 MB |

The R-N0-4 rebuild floor (≥15k events/s at 50k) **holds with wide margin** —
49.0k ev/s, ~3.3× the floor (2026-08-11: 54.6k ev/s — this run reads ~10%
lower, still comfortably clear; both are the same order). Fixed startup
overhead ~264–300 ms (matches 2026-08-11's ~262–269 ms). Cold-query scaling:
`blocked_time_per_work` cold calls run 79.6 → 216.4 → 409.5 ms at
10k/25k/50k (2026-08-11: 70.6 → 227.9 → 401.8 ms — same shape, same order).
~635 B/event steady (2026-08-11: ~571 B/event — larger events, consistent
with MVP-1's manifest/envelope fields riding on the same event kinds; no
new event kinds were needed to carry them, matching A3's "core resolves and
pins, adapters translate without redefining" ruling). DuckDB on-disk:
33.0 MiB after 50k (2026-08-11: 32.0 MiB — directional, in line with more
fields per row).

## S6 — kill -9 mid-burst recovery, ×3 cycles

Across 3 cycles, 10 concurrent submits in flight at kill time: **zero
silently lost works, zero illegal states, zero duplicate executions, zero
orphan processes**, every cycle; second restart byte-identical each time
(`c{1,2,3}_second_restart_idempotent: yes`); `doctor healthy: true`,
0 unparseable journal lines. 31 works total after 3 cycles (24 blocked, 7
completed). Per-scenario hygiene reads `dirty` this run (2 populated + 22
empty leftover surface dirs, `worktrees: 2`) versus 2026-08-11's `clean`
(23 empty, 0 registered worktrees) — both are the same documented shape
(a non-terminal `blocked` work keeps its surface until it goes terminal, not
a leak); this run's kill timing happened to catch 2 works with an already-
materialized worktree instead of 0, which is expected stochastic variance
in exactly when the kill lands relative to surface creation, not a new
defect. The core recovery invariants (zero loss / zero illegal states / zero
duplicate executions / zero orphans) hold identically to 2026-08-11.

## S7 — client hygiene under load

| metric | value |
|---|---|
| TUI idle CPU (30 s, live SSE) | 0.4% (2026-08-11: 0.3%) — still no busy loop |
| TUI RSS | 14.8 MB idle, +0 kB over idle window, +376 kB through burst |
| daemon fds with TUI attached | 15 → 17 → 15 after quit |
| `q` exit | 235.3 ms |
| `sgt status` / `sgt web` | 4.3 ms / 4.5 ms under load |
| `sgt doctor` | 524.3 ms (2026-08-11: 240.8 ms) |

`sgt doctor`'s ~2.2× rise is consistent with MVP-3 landing doctor **estate
checks with named remedies** (new work the 2026-08-11 binary did not do) —
directional, not flagged as a regression against any stated budget (none
exists for `doctor`). The orphaned-TUI SIGTERM finding continues to hold
refuted (`orphan_repro: "refuted: SIGTERM reaped it within 2s"`,
1088.67 ms) — same fix (`de193a2`) still in effect.

## Budget lines re-baselined (R-N0-4, `docs/gauntlet/contracts/N0.md`)

| Budget | Bound | 2026-08-11 (pre-MVP) | 2026-08-12 (assembled MVP-1..3) | Flag |
|---|---|---|---|---|
| Submission throughput, burst 50 | ≥28 works/s | 42.0 works/s | **41.4 works/s** | holds, wide margin |
| Single-submit e2e p50 | ≤50 ms | 41.3 ms (single sample) | **25.7 ms** (single sample) | holds |
| Core-lock discipline | no ext. I/O / proc wait / join under lock | not re-verified (timing harness only) | not re-verified (same) | — out of scope |
| Execute-capture memory | peak RSS incr ≤64 MiB / 1 GiB capture | not exercised (fake backend) | not exercised (same; Docker execute stage exists but this harness's non-goals exclude driving it — R1) | — |
| Rebuild rate | ≥15k events/s @50k | 54.6k ev/s | **49.0k ev/s** | holds, ~3.3× floor |
| Journal cost per execute stage | O(1) events/stage | 18.0 events/work | **18.0 events/work**, S1+S2 | holds, unchanged |

**Verdict: the assembled MVP-1..3 product does not regress any R-N0-4
budget.** The one scenario that changed shape (S3's depth) changed because a
new MVP-1 safety mechanism (the turn envelope) now legitimately intervenes
on the harness's own long-running fake script — not because anything got
slower or leakier. The one un-budgeted directional finding worth carrying
forward is S2's higher per-work RSS retention (¶ above).

## Coverage

Ran cleanly end to end per `scripts/coverage/` (C0→C4, `COV_ARTIFACTS`
pointed at a fresh `docs/coverage/artifacts-2026-08-12/` dir, kept separate
from the committed 2026-08-10 S1-COVERAGE baseline's evidence tree so this
run cannot corrupt that record — dev profile per the harness's own
convention, diverging from this doc's release-profile perf numbers by
design). C0 profraw-pattern check passed; C1 (`--lib`, 236 tests), C2
(m1/m4/m3/m5), C3 (m2/m6, the spawning suites) all green, zero lost
profraws at every stage; C4 merged 122/122 profraws.

| metric | value |
|---|---|
| Lines | **89.36%** (18,771 instrumented; 1,998 missed) |
| Regions | 89.40% (30,299 total; 3,213 missed) |
| Functions | 88.22% (1,630 total; 192 missed) |

Recorded honestly, not chased: this is a directional read against the
2026-08-10 S1 baseline (91.43% lines, 10,466 instrumented), not a strict
apples-to-apples trend line — the instrumented line count nearly doubled
(10,466 → 18,771) as MVP-1..3 landed the manifest/estate/envelope/CLI/Docker
code, and the ~2-point drop is concentrated in the newest, least-tested
areas rather than spread evenly: `backend/docker.rs` 42.97% lines (N4's
Docker executor — the coverage harness's fake-backend-only suites don't
drive a real container), `cli.rs` 66.43% (MVP-3's new verbs), `backend/mod.rs`
88.81%. Everything else sits at or above the 2026-08-10 baseline's weakest
files. No fix attempted here — S1-COVERAGE's own contract scopes this
harness to measurement, and this task's brief is explicit that coverage is
record-only. Full per-file table: `docs/coverage/artifacts-2026-08-12/c4-summary.txt`;
lcov/html at `target/llvm-cov-reports/` (uncommitted, regenerable).

## Hygiene

Per-scenario sweeps: idle/s1/s2/s4/s7 `clean`; s3/s6 `dirty` — both
explained above as non-terminal-work surfaces, not leaks. Zero `sgt`
processes leaked in any scenario (`hygiene_s*_sgt_processes: 0` throughout);
`pgrep -f "debug/sgt [-]-data-dir"` and `release/sgt [-]-data-dir"` both
empty after the run. The aggregate final sweep (`run-all.sh`'s own
end-of-run `perf_hygiene final`, which does not carry `PERF_DATA_DIR` and so
cannot distinguish populated-vs-empty surface dirs the way per-scenario
sweeps do) reads `dirty — worktrees=3`: the same 3 non-terminal-work
worktrees (1 from S3, 2 from S6) counted again at the aggregate level, not a
new or different leak. `/tmp` residue held flat at 49 entries across every
scenario (host-level `.tmp*` files present before this run started, not
created by it — unchanged pre/post). All scenario data dirs and repos live
under the scratch outdir, outside the repo tree; repo tree stayed clean for
the whole run (`git status --porcelain` empty, confirmed both before and
after).

## Anomalies

None. Every scenario's self-reported `commit` field agrees
(`aedc7cb075d573854a7986a0b6d9078c4cf730e7`), `commit_dirty_tree=0`, and
`binary_swapped_since_pin=0` throughout — the 2026-08-11 baseline's
concurrent-commit-drift anomaly (issue #50) does not reproduce here.
