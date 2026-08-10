# N3 — what the two-phase boundary cost, measured

Governing: R-N0-4's regression budgets (`docs/gauntlet/contracts/N0.md`),
against `docs/perf/baseline-2026-08-10.md`. Harness: `scripts/perf/s1-burst.sh`
at `PERF_S1_BURSTS="1 50" PERF_S1_SETTLE=2`, release binary, same container as
the baseline, three runs per variant, back to back.

**Headline: one budget holds, one is breached and the breach is diagnosed.**

| budget (R-N0-4) | bound | measured | verdict |
|---|---|---|---|
| single-submit e2e p50 | ≤ 50 ms | **39.3 ms** (39.3 / 39.4 / 39.6) | holds |
| submission throughput, burst 50 | ≥ 28 works/s | **24.5 works/s** (24.2 / 24.3 / 24.9) | **breached** |
| core-lock discipline | no external I/O, process wait or thread join under the lock | §22.6 tests t9/t10, unrelated requests answered in ~2 ms while an executor was parked | holds |

## The control

The baseline document's numbers were taken on this container *class*, not this
container on this day, so the comparison that matters is an A/B on the same
machine minutes apart. `fdbadcd` (N3 Outcome 2, the commit before the two-phase
boundary) was built release and measured the same way:

| variant | burst-50 throughput | burst-50 p50 | events/work |
|---|---|---|---|
| `fdbadcd`, pre-boundary (control) | 35.0 / 32.9 / 32.9 works/s | 647 / 708 / 721 ms | 16.0 |
| N3, `spawn_blocking` per launch | 20.4 / 21.5 / 22.0 works/s | 2183 / 2059 / 2014 ms | 18.0 |
| N3, `block_in_place` per launch | 27.1 / 26.9 / 25.2 works/s | 1586 / 1593 / 1685 ms | 18.0 |
| N3, `block_in_place` + guard reuse (shipped) | 24.3 / 24.9 / 24.2 works/s | 1711 / 1632 / 1720 ms | 18.0 |

So the control itself sits at ~33 works/s today, under the 37.6 the baseline
recorded — this container is a little slower than the baseline host was. The
regression attributable to N3 is therefore ~33 → ~25, about −25%, not the −35%
a naive read against the published baseline would suggest.

## Where it goes

Two costs, both structural rather than incidental:

1. **Two more fsynced journal appends per work** — 16 → 18 events/work, exactly
   one `execution.reserved` per stage entered, for a two-stage workflow. That
   is +12.5% of appends on a path the baseline already identified as
   single-writer-bound ("throughput plateaus at ~28–39 works/s regardless of
   concurrency … the daemon saturates first"). Per-event cost is essentially
   unchanged (1.88 ms control vs 2.07 ms), so most of the wall-clock delta is
   simply *more events*.
2. **Three authoritative phases instead of one** — a submit now takes the core
   mutex to reserve, again to settle stage 0 and reserve stage 1, and again to
   settle stage 1. Tokio's mutex is FIFO-fair, so under a 50-deep queue each
   work's end-to-end latency includes three queue traversals; that is the whole
   of the p50 move (647 ms → ~1700 ms ≈ 2.6×, against 3× the acquisitions), and
   it is the *intended* shape: the queue drains while a harness is spawning
   instead of behind it.

Both are the price of the boundary, not a defect in it. The first is the
journal record the crash matrix reads (§22.5 windows 2–4 exist *because*
`execution.reserved` is durable); the second is what §22.6 is asking for.

## What was tried, and what it bought

- **`spawn_blocking` → `block_in_place`** for the launch and for draining an
  adapter completion: +24% throughput (21.3 → 26.4 mean). `spawn_blocking`
  moves the value to a pool thread and back, two task hops per launch and 100
  launches per burst; `block_in_place` runs the closure on the current worker
  and hands that worker's other tasks to a replacement. Shipped.
- **Returning the guard from `crank`** so a handler renders its response
  without queueing a fourth time: single-submit p50 improved (50.0 → 39.3 ms,
  which is the budget that is actually binding at burst 1), burst-50
  throughput moved −7% (26.4 → 24.5 mean), inside the spread of a three-run
  sample. Shipped for the p50, and because fewer acquisitions is the
  structurally cheaper shape; a larger sample may want this re-adjudicated.
- **Merging `stage.entered` and `execution.reserved` into one compound event**
  (L6's other answer, and it would return events/work to 16): *rejected*. §22.5
  enumerates "before reservation append" and "immediately after reservation
  append" as two distinct injection windows, and the contract's matrix — and
  `n10`/`n11` — depend on them being separable. Buying throughput by deleting
  a crash window is not a trade this milestone may make.

## The recommendation this hands forward

The remaining gap is fsync volume on a single-writer path, and the fix that
addresses it is **group commit inside one lock hold**: `Core::commit` currently
fsyncs per event, and every burst-50 work appends 18 of them in three
contiguous runs. Batching the fsync per lock hold would recover more than the
reservation costs, and it is a journal change with its own crash semantics to
design and test — properly its own scope, not a line item smuggled into the
execution boundary. Filed here rather than fixed here.

Until then the burst-50 throughput budget stands breached at ~24.5 works/s
against a ≥28 floor, on the record, with the cause named.
