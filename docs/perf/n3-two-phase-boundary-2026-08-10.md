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

---

# Wave-2 addendum (2026-08-10, after the review round's fixes)

Same harness, same container, same day, release binary rebuilt from the fix
series; three runs, `PERF_S1_BURSTS="1 50" PERF_S1_SETTLE=2`.

## The core-lock verdict above was overstated (INV-N3-08)

Row 3 of the headline table read "**holds**" on the strength of t9 and t10, and
per L9 that verdict is itself gradeable. It was a claim about two code paths
reported as a claim about the lock. The instrument could park the fake inside
`launch` and inside a stop `Completion` and nowhere else, so it was structurally
incapable of seeing the three external effects that were still under the guard
when it returned "holds":

- `git rev-parse`/`symbolic-ref`/`worktree add`/`worktree remove` per
  repository, on submit, retry and cancel (INV-N3-02 — measured at 86 ms for
  one `worktree add` on a 3.4 MB `.git` in this container);
- `Command::spawn("claude" …)` plus three thread spawns on every `respond`
  (INV-N3-03 — SEND was never split);
- `ClaudeBackend::observe` walking `/proc` for a restart-classified handle.

The correct wave-1 verdict was **"holds for LAUNCH and for the STOP archive
join; unmeasured elsewhere"**. It is recorded here rather than edited above:
the table is what was reported, and this is what was wrong with it.

## What the instrument is now

Timing tests can only cover the effects someone thought to gate, so the budget
is no longer defended by timing alone:

| instrument | covers |
|---|---|
| m6 `t11_external_effects_live_only_in_the_out_of_lock_performers` | **the class.** Every `materialize`/`rematerialize`/`teardown`/`backend.launch`/`backend.send` call site in `runtime/engine.rs` must lie inside a performer that takes no `Core`, with the two single-owner exceptions named in the test. Moving one back into a `&mut Core` path fails here with the call site printed. |
| m2 t9 / t10 / **t11** | LAUNCH, the STOP archive join, and now SEND (the fake grew a send gate). |
| m4 n19 / n20 / n21 | the git phases: no worktree exists when `begin_start` returns; the worktree still exists when `begin_retire_run` returns; a cancel landing mid-`git` records the surface and tears it down. |
| m2 `INDEPENDENT_REQUEST_BUDGET` | 1 s → **200 ms** (N3-08: the old bound was ~500× the value this document reports). |

## The throughput breach is closed

| budget (R-N0-4) | bound | wave 1 | wave 2 | verdict |
|---|---|---|---|---|
| single-submit e2e p50 | ≤ 50 ms | 39.3 ms | **42.9 ms** (39.2 / 43.7 / 45.7) | holds |
| submission throughput, burst 50 | ≥ 28 works/s | 24.5 (breached) | **32.7 works/s** (33.6 / 32.6 / 31.8) | **holds** |
| core-lock discipline | no external I/O, process wait or thread join under the lock | "holds" — overstated, see above | holds, on the instruments above | holds |

The same-machine control (`fdbadcd`, pre-boundary) measured 33.0 works/s in
wave 1. At 32.7 the two-phase boundary now costs **≈1%** of burst throughput
rather than 25%, with the two extra fsynced events per work unchanged at 18.0.

Nothing was traded away for it. The +2 events/work the wave-1 breach was
attributed to are still there, and `stage.entered`/`execution.reserved` are
still two appends (the merge stays rejected: §22.5 enumerates their two sides as
distinct injection windows). What changed is what else was holding the writer:
submission no longer discovers the workspace, reads every stage's `CONTEXT.md`,
probes harnesses or runs three `git` processes per repository under the core
mutex, and neither cancel nor retry runs git under it. The single-writer path
the P1 baseline identified as the bottleneck is now doing only journal work.

**Consequence for the A-N3-1 ruling.** The amendment lowered the burst-50 floor
to ≥24 works/s for this milestone because the boundary breached ≥28. The breach
is gone, so the amendment is no longer load-bearing — the original R-N0-4 floor
is met with ~17% headroom. That is a fact for the panel to rule on, not a licence
to edit the ruling: A-N3-1 also filed **#44 (journal group commit) with a hard
trigger, "lands before the N4 contract ships"**, and that trigger was justified
by Docker's added reserved-event volume rather than by this milestone's number.
It should survive the amendment it came with.

## A bug the fix uncovered, and its guard

Moving git off the core lock removed the accidental serialization the core lock
had been providing, and a burst of 50 submissions promptly ran
`git worktree add`/`remove` against one `.git` from many threads at once. Git
does not serialize that: two teardowns in the first wave-2 run failed with
`fatal: failed to read .git/worktrees/<other-work>/commondir` — one process
walking the worktree registry while another rewrote it — and, failing closed as
designed, *retained* those worktrees. Honest, journaled, and still two surfaces
left on disk out of 51 (`hygiene_s1_verdict: dirty`).

`runtime::surface` now takes a per-source-repository lock across each worktree
mutation. It is not the core lock, it blocks no request, and two works in
different repositories still proceed in parallel. The three runs above are
`hygiene_s1_verdict: clean`, zero surface dirs. Pinned by
`concurrent_surfaces_on_one_repository_all_materialize_and_retire_cleanly`,
which fails 5 runs in 8 with the lock removed and passes deterministically with
it — the same shape the bug had.

---

# Round-2 addendum (2026-08-10): the blind band these instruments leave

Recorded, not fixed. Round-2 finding N3R2-05 measured what the tightened
budgets actually catch, and the honest answer has a hole in the middle of it
that a reader of the tables above would not guess.

## The per-lock-hold band: anything under ~200 ms

`INDEPENDENT_REQUEST_BUDGET` went from 1 s to 200 ms in wave 2, and that change
has real teeth: with a 300 ms blocking sleep under the submit guard, m2's t9
and t11 both fail with the intended message, where the old 1 s bound would have
passed it without a murmur. But the value the boundary was moved for is 86 ms —
the measured cost of one `git worktree add` on a 3.4 MB `.git` in this
container, i.e. INV-N3-02 itself — and with an 86 ms sleep under that same
guard **nothing in the tree fails**. The §22.6 instruments see a hold of 200 ms
or more; below that they see nothing, however many times per second it happens.

The throughput floor does not close the band either, and it is a slower
instrument than it looks: m2's t12 (rewritten in round 2 to submit on the real
path) catches any effect of duration *d* serialized under the guard once
`1/d` drops below its 12 works/s floor, i.e. from about **80 ms** upward. Below
that the wall-clock signal is inside the run-to-run spread of a shared test
host. A second fsync per journal append — A-N3-1's own cost story — is ~5% of a
submit here (38.2 → 36.8 works/s), which is to say invisible.

So, stated plainly: **the contract's Budgets section is blind to any
per-lock-hold regression shorter than ~200 ms, and to any per-submit cost
smaller than ~80 ms, no matter how often it is paid.**

## Why it is not simply tightened

These suites run in parallel with seven others on shared cores. A 20 ms
independent-request budget would fail on a scheduler hiccup rather than on a
regression, and a flaky budget does not get investigated, it gets deleted —
which is the failure mode L7's corollary describes from the other side. The
200 ms figure is two orders of magnitude above the ~2 ms a contended journal
commit actually costs here and two orders below the failures it was written to
catch; the band between "measurably wrong" and "provably fine" is the price of
not flaking.

## What stands in the band's place

Nothing timing-based, deliberately. The instruments that cover the small-effect
case are structural and do not measure duration at all, so a 1 ms effect trips
them exactly as readily as a 900 ms one:

| instrument | sees |
|---|---|
| m6 `t11_external_effects_live_only_in_the_out_of_lock_performers` | any `materialize`/`rematerialize`/`teardown`/`launch`/`send`/`observe`/`resume` call site that moves into a `&mut Core` path, at any cost |
| m6 `t11b_the_append_path_issues_exactly_the_fsync_it_accounts_for` | a second durability syscall per append, at any cost |
| m4 n19 / n20 / n21 | the git phases, by what exists on disk when a phase returns rather than by how long it took |

The timing tests are the backstop for effects nobody thought to enumerate. They
are not, and after this measurement should not be described as, the primary
defence of the §22.6 budget.
