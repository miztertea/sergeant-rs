# Performance Baseline — Cerberus (2026-08-11)

First non-container baseline, per the same [P1-PERF contract](../gauntlet/contracts/P1-PERF.md) harness (`scripts/perf/`) used for `docs/perf/baseline-2026-08-10.md`. Every number below comes from the raw run artifacts (`run-all-status.tsv`: all 8 stages `ok`) — nothing is estimated; unsourced cells are marked "—". Numbers are host-relative, same convention as the container doc: comparisons below are directional, not absolute-hardware claims.

**Unit under test:** `target/release/sgt`, commit `f9b047da077a8f525ea00ff1a33ffe1733e8f897` (`s1-burst-summary.json`'s `commit` field; matches `environment.txt` and `sgt --version`; binary mtime confirms one build, right after that commit, never rebuilt during the run — see Anomalies). **Environment:** Cerberus host, 20 cores (bare metal, no cgroup quota, `nproc` 20), 32,058,072 kB RAM, `Linux 7.0.0-29-generic`, loopback HTTP, fake backend except where noted. First measurement off the container class.

**Caveat (recorded for honesty, not adjusted for):** a live user-installed `sgt` daemon (pid 35846, `~/.cargo/bin/sgt`, data dir `~/.local/share/sergeant`) was idle-resident throughout the run. Every scenario here uses its own daemon and data dir, so contamination is limited to background noise on a 20-core box.

## Idle profile

| metric | value |
|---|---|
| RSS (119×1s samples, 120s settle) | 29.9 → 29.8 MB, drift −16 kB (flat) |
| fds / threads | 15 / 41 |
| CPU over settle window | 0.000 s |
| cold start, empty data dir | 279.39 ms (single sample) |

Threads (41) exceed the container's 9 — a tokio pool sized to `nproc` (20 vs 4), not a leak.

## S1 — burst submissions (concurrent `POST /v1/work`, two-stage workflow)

| burst | wall | throughput | p50 | p95 | max | RSS peak | fds peak | container tp (pre-N3) |
|---|---|---|---|---|---|---|---|---|
| 1 | 0.084 s | 11.9/s | 41.3 ms | — (1 sample) | 41.3 ms | 30.5 MB | 15 | 17/s |
| 5 | 0.125 s | 40.0/s | 87.1 ms | 96.9 ms | 97.7 ms | 31.8 MB | 18 | 31.8/s |
| 10 | 0.218 s | 45.9/s | 163.4 ms | 178.9 ms | 179.0 ms | 32.7 MB | 27 | 28.8/s |
| 20 | 0.442 s | 45.3/s | 364.8 ms | 397.4 ms | 399.0 ms | 35.2 MB | 37 | 35.6/s |
| 50 | 1.191 s | 42.0/s | 1013.9 ms | 1086.8 ms | 1094.3 ms | 40.8 MB | 67 | 37.6/s |

All 50 works confirmed completed at every cell; 18 events/work uniformly (N3 two-phase-boundary shape, not P1's 16). At burst 1 (single sample both sides) Cerberus reads below the container; everywhere with >1 sample it clearly outruns it. Single-submit p50 (41.3 ms) holds the ≤50 ms R-N0-4 budget, same order as the N3 addendum's 39.3/42.9 ms. Burst-50 throughput (42.0/s) beats both the container's N3 wave-1 breach (24.5/s) and its wave-2 fix (32.7/s) — see Budget lines for what that does/doesn't mean.

## S2 — sustained churn, 200 works in waves of 10, 120 s settle

| metric | value |
|---|---|
| submit phase wall (20 waves) | 4.861 s |
| RSS pre → peak → post-settle | 30.1 → 37.3 → **37.3 MB (zero reclaim)** |
| RSS per completed work (regression slope) | **25.88 kB/work, monotonic non-decreasing** |
| fds pre/peak/post | 15 / 27 / 15 |
| daemon CPU per work | 2.5 ms |
| events per work | 18.0 |

The ~25 kB/work no-reclaim shape reproduces almost exactly (container: ~25 kB/work, slope +21 kB/work) — same code, different host: it's the retained projection, not an environment artifact.

## S3 — deep work (1,011 events on one work) + graph read load

| metric | value |
|---|---|
| graph response at depth | 3,033 B (9 nodes / 8 edges) — same shape as container |
| respond cycles (200×) | p50 22.2 ms, p95 25.0 ms |
| sequential 100× graph reads | p50 14.1 ms, p95 16.3 ms |
| 20-way concurrent ×5 rounds | p50 31.0 ms, p95 39.9 ms (~2.2× amplification) |
| round wall (20 reqs) | 0.057–0.063 s across 5 rounds |
| RSS before/peak/after | 40.0 → 89.4 → 57.6 MB (+17.5 MB retained, single-sample) |
| fds | 15 / 18 / 15 |

Amplification is lower here (~2.2×) than the container's ~4.4×, consistent with less core oversubscription for the same 20-way burst (20 vs 4 cores). Retained-RSS echoes the container's +13 MB single-sample finding — same caveat: needs a repeat-trend before it's called anything more.

## S4 — SSE fan-out: 25 subscribers through a 20-work burst

Exactly 1 fd per subscriber (15 → 40, matches container). Phase 1: burst wall 0.487 s, submit p50 407.6 ms / p95 442.0 ms, 25/25 subscribers saw all 20 terminal events. Phase 2: 10 subscribers killed mid-flight, burst wall 0.454 s, 20/20 HTTP 201, 15/15 survivors saw all events, a post-kill submit completed normally (not wedged). fds returned exactly to 15, 0 leaked. No without-subscriber A/B was captured here, so the container's "~35% latency rise, refuted" claim has no Cerberus comparison point — left as "—".

## S5 — journal scale: cold-start rebuild + analytics

| mark | cold start | rebuild rate (raw) | RSS after start | journal size |
|---|---|---|---|---|
| 10k events | 357.55 ms | 28,958 ev/s | 67.3 MB | 5.6 MB |
| 25k events | 535.89 ms | 47,040 ev/s | 107.0 MB | 13.7 MB |
| 50k events | 924.09 ms | 54,553 ev/s | 156.9 MB | 27.5 MB |

The R-N0-4 rebuild floor (≥15k events/s at 50k) **holds with wide margin** — 54.6k ev/s, ~1.9× the container's 29.2k. Fixed startup overhead is ~262–269 ms (container: ~425–450 ms — lower, faster hardware). Cold-query scaling reproduces: `blocked_time_per_work` cold calls run 70.6 → 227.9 → 401.8 ms at 10k/25k/50k (container: 153 → 495 → 792 ms, same shape, ~half the absolute cost). ~571 B/event steady (container ~549 B/event). DuckDB on-disk: 32.0 MiB after clean shutdown at 50k (container: 31.8 MiB).

## S6 — kill -9 mid-burst recovery, ×3 cycles

Across 3 cycles, 10 concurrent submits in flight at kill time: **zero silently lost works, zero illegal states, zero duplicate executions, zero orphan processes**, every cycle; second restart byte-identical each time; doctor healthy; 0 unparseable journal lines. 31 works total after 3 cycles (23 blocked, 8 completed). Hygiene verdict "clean" despite 23 empty leftover surface dirs — expected, since a non-terminal (blocked) work keeps its surface until it goes terminal, not a leak. The container's finer-grained L6-class observation (trailing events lost to the crash window on some completions) isn't checkable from these artifacts — "—".

## S7 — client hygiene under load

| metric | value |
|---|---|
| TUI idle CPU (30 s, live SSE) | 0.3% (container: 0.03%) — still no busy loop |
| TUI RSS | 13.7 MB idle, +64 kB over idle window, +432 kB through burst |
| daemon fds with TUI attached | 15 → 17 → 15 after quit |
| `q` exit | 234.4 ms |
| `sgt status` / `sgt web` | 4.4 ms / 3.3 ms under load |
| `sgt doctor` | 240.8 ms (container: ~450 ms floor) — same load-independent shape, lower absolute cost |

The container's critical finding — an orphaned TUI (pty destroyed) ignores SIGTERM, needs SIGKILL — **does not reproduce here**: SIGTERM reaped the orphan in 1114.66 ms (`orphan_repro: "refuted: SIGTERM reaped it within 2s"`). Not a new divergence: `de193a2 tui: exit cleanly when the pty dies; never park shutdown on the reader` landed between the container baseline's commit and this one. This run reconfirms the fix, it doesn't find a new gap.

## Budget lines re-baselined (R-N0-4, `docs/gauntlet/contracts/N0.md`)

| Budget | Bound | Container | Cerberus | Flag |
|---|---|---|---|---|
| Submission throughput, burst 50 | ≥28 works/s | 37.6 (P1) / 24.5 (N3 w1, breached) / 32.7 (N3 w2, holds) | **42.0 works/s** | holds, wide margin |
| Single-submit e2e p50 | ≤50 ms | 37.9 ms (P1) / 39.3–42.9 ms (N3) | 41.3 ms (single sample) | holds |
| Core-lock discipline | no ext. I/O / proc wait / join under lock | holds (structural m6 t9–t11) | not re-verified (this harness measures timing, not lock structure) | — out of scope |
| Execute-capture memory | peak RSS incr ≤64 MiB / 1 GiB capture | not exercised (fake backend) | not exercised (same) | — |
| Rebuild rate | ≥15k events/s @50k | 29.2k ev/s | **54.6k ev/s** | holds, ~1.9× container |
| Journal cost per execute stage | O(1) events/stage | 18.0 events/work (post-N3) | 18.0 events/work, S1+S2 | holds, unchanged |

**On #44 (journal group commit):** burst-50 here (~42/s) beats even the container's post-fix wave-2 number (32.7/s) — but per the N3 addendum, #44's trigger was justified by N4/Docker's added `execution.reserved` volume, not by any throughput shortfall on this milestone's number. A faster host clearing today's 18-events/work load easily says nothing about headroom under N4's larger volume. This measurement doesn't retire #44.

## Anomalies

**Self-reported `commit` field drifts across scenarios, unreliable past S2.** `idle`/`s1`/`s2` record `f9b047d…`; `s3`/`s4`/`s5` record `5d3b333…`; `s6`/`s7` record `81d83baf…`. All three are real commits on this checkout's `main` (`git log f9b047d..HEAD` lists exactly them, landed 06:02:52, 06:04:21, 06:08:14 — during the ~7-minute run, from a concurrent session committing to the same tree). The binary did **not** change: `target/release/sgt`'s mtime (Aug 11 05:58) matches right after `f9b047d` (05:57:35) and was never touched again. So every number here is against one binary at `f9b047d` as stated above — but the harness's `commit` field is a live `git rev-parse` per scenario start, not a pin to the tested binary, and reads wrong for 6 of 8 scenarios. Worth fixing before the next non-container run.

