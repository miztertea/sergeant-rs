# Issue #44 — what the journal group commit bought, measured on Cerberus

Governing: issue #44 (filed by A-N3-1), `docs/perf/n3-two-phase-boundary-2026-08-10.md`
(the regression it answers), `docs/perf/baseline-cerberus-2026-08-11.md`
(this host's baseline). Budget line: R-N0-4's burst-50 floor, ≥28 works/s.

Harness: `scripts/perf/s1-burst.sh` at `PERF_S1_BURSTS="1 50"
PERF_S1_SETTLE=2`, release binary, fake backend, three runs per variant, back
to back, same host, same hour. Host facts: `docs/environments/cerberus.md`
(20 cores, ext4 on LVM, kernel 7.0.0-29).

## Headline: the fsyncs are gone; the wall clock barely notices, and that is the finding

| variant | burst-50 throughput | mean | burst-50 p50 | events/work |
|---|---|---|---|---|
| before (`1f0b6c2`, per-append fsync) | 43.59 / 43.57 / 42.23 works/s | **43.13** | 962 / 973 / 1019 ms | 18.0 |
| after (group commit) | 42.20 / 46.22 / 45.44 works/s | **44.62** | 1008 / 906 / 926 ms | 18.0 |

**+3.5% mean, inside the run-to-run spread of both samples.** Single-submit
p50 stayed inside budget throughout (before 45.8 / 27.6 / 32.9 ms; after
37.8 / 44.5 / 46.6 ms — one sample per run, so the spread is the instrument,
not the change). Events per work is 18.0 on both sides, which is the point:
no event was merged, added or removed.

The syscall count is where the change is unambiguous. Same harness, one
burst-50 run per variant, daemon wrapped in `strace -f -c -e
trace=fdatasync,fsync`:

| variant | `fdatasync` calls | per work | per event |
|---|---|---|---|
| before | **1157** | 23.1 | 1.29 |
| after | **253** | 5.1 | 0.28 |

904 fsyncs removed for 900 journaled events — i.e. exactly the per-event
fsync, collapsed onto ~5 group boundaries per work (the submit hold, the
launch settle, the two stage settles, the command record). Measured cost of
the ones that remain: 34 µs/call after vs 13 µs/call before, total fdatasync
time 8.8 ms after vs 15.1 ms before across the whole burst. The surviving
syncs are more expensive each because each one now flushes more bytes; they
are still a third of the total time.

## Why 4.6× fewer fsyncs is worth 3.5% of wall clock

Because on **this** host an fsync is nearly free and was never the
bottleneck. 15.1 ms of fdatasync across a 1.15 s burst is 1.3% of the wall
clock; removing three quarters of it cannot buy more than ~1%, and the rest
of the measured +3.5% is noise. Cerberus writes to ext4 on an LVM volume
with a write cache in front of it; the container the N3 regression was
measured on behaved the same way (round-2 finding N3R2-04 measured a second
fsync per append there at ~5% of a submit, "invisible" to any floor that
does not flake).

So this document does **not** claim #44 recovered the N3 regression on
Cerberus. There was no regression left to recover here: the N3 wave-2 fix
(git and harness probing off the core lock) already put burst-50 at 32.7
works/s on the container and this host measures 42–45 works/s against a ≥28
floor. What #44 changes is the *shape* of the cost, and the shape is what
the trigger was about — the `baseline-cerberus` note said it plainly:

> A faster host clearing today's 18-events/work load easily says nothing
> about headroom under N4's larger volume. This measurement doesn't retire
> #44.

Journal cost per work is now **O(lock holds), not O(events)**. N4's Docker
execute stages add `execution.reserved` volume per stage; under per-append
fsync that volume was linear in syscalls, and it is now free within a hold.
The number to carry forward is 5.1 fsyncs/work, not 44.6 works/s.

### The honest negative

If a future host has a real fsync cost — no write cache, a network
filesystem, `data=journal` — the before/after gap widens by construction and
this measurement understates the change. Nobody has measured that host. The
claim here is bounded to Cerberus, and the syscall counts are the portable
part of it.

## Ambient load, stated

Cerberus was not idle. A harvest workflow's Sonnet actors run on this host
(`pgrep -c -f claude` = 4 throughout both variants, mostly API-bound) and
the `sergeant-harvest` checkout runs its own legitimate daemon — it is what
makes every run in this document report `hygiene_s1_verdict: dirty` with
`hygiene_s1_sgt_processes: 1`, on both sides of the change. Load average
`/proc/loadavg` at run start: 0.29 before the "before" reps (rising to 3.45
mid-series), 1.64 before the "after" reps (1.18 at the end). Every other
hygiene counter is 0 on all six runs: zero leaked worktrees, zero surface
dirs, zero `/tmp` residue, 50/50 works completed and confirmed each time.

The "after" reps ran under the *higher* ambient load of the two, which if
anything biases the measured +3.5% downward. That is not offered as a
correction — three runs a side cannot separate a 3% effect from a 3% load
difference, and the honest summary is "no measurable throughput change on
this host at this volume."

## Raw artifacts

**Correction, 2026-08-11 (round-2 finding INV-R2-07).** This section
originally cited the six `s1-burst-summary.json` files (3 before, 3 after)
plus the two strace summaries as living "under the session scratchpad".
That location does not survive this environment — CLAUDE.md's
remote-container section states these containers reset without warning and
wipe installed tools and `target/`, and a scratchpad path is session-scoped
on top of that — so by the next session the artifacts were already gone,
and confirmed gone when checked for this correction: no matching files exist
anywhere under this checkout, and none were ever committed. L11's rule is
that an integrity claim binds only when its verification procedure is
executable by a stranger; citing an ephemeral path failed that silently.
**The transcribed table above is therefore the only surviving evidence for
this measurement** — a stranger cannot re-derive it from raw data, only
re-run the harness and compare. Numbers above were transcribed from
`b50_throughput_per_s`, `b50_lat_p50_ms`, `b50_events_per_work` and
`hygiene_s1_*`; nothing was estimated, but nothing here proves that anymore.
Going forward, raw harness output for a measurement this document depends on
belongs committed under `docs/perf/` alongside the note that cites it, not
left in a scratchpad.

**Instrument caveat, inherited.** The baseline document's Anomalies section
records that the harness's self-reported `commit` field is a live `git
rev-parse` per scenario, not a pin to the binary under test. It is right in
these runs (nothing else committed to this checkout while they ran), but it
is right by luck, not by construction.
