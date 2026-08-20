# Rule C — Journal Segment Archival, Blob Liveness, and Derived-Store Growth

Status: Proposed — awaiting owner ratification
Date: 2026-08-20
Audit basis: `miztertea/sergeant-rs` `backlog/w5-archival-design` @ `a730983c`
(integration branch carrying waves 1–2 of the backlog close-out sprint)
Scope: Journal segment retention, the replay contract's floor, blob-store
liveness, DuckDB growth, `sgt doctor`'s growth axis, and the explicit
maintenance verb
Product behavior: Changed — the journal gains an archive tier and a
non-zero replay floor; nothing is deleted, and no automatic action is
added

────────

## 0. Relationship to existing decisions and defects

| Existing decision or defect | Disposition here |
|---|---|
| `docs/DEVELOPMENT.md:37`: "the journal is the only truth… rebuild-on-start is the only population path; there is no snapshot loading (backlog B1 explains why)" | **Preserved, with one narrow amendment proposed.** Archived segments stay inside the replay contract. The only derived state this proposal persists is the slim index row (§4.3) — seven scalar fields per Work, all re-derivable, binding-checked against the archive. B1's general snapshot machinery stays dormant. |
| ADR 0003 (durability promise) | **Not amended by the recommended composition.** ADR 0003 promises *resumability*, not never-delete (§2.6). A rung that discards history would amend it; none of the adopted rungs do. The tripwire is pre-committed in §2.6 so a later rung cannot cross it silently. |
| Retention ruling, 2026-08-11 (Rule C, amended at adjudication): binding trigger is rebuild-on-start > 30 s; ladder is compress-cold-segments-first, then snapshot/truncate | **Honored, with one correction offered for ruling.** Compression is adopted — as a rider on archival rather than as a standalone rung — but §3.5 shows compression addresses *disk*, while the ruled trigger is *time*. A cheaper rung 0 (§4.1) sits below both. Open question Q1. |
| Retention ruling, Rule B: `sgt doctor` disk-pressure check | **Extended.** The shipped check (`src/cli.rs:3018`, thresholds at `3042-3043`) measures *free space*, not *growth* — it cannot see a data dir growing to 500 GB on a disk with 600 GB free (§3.4). A growth axis is added beside it. |
| Retention ruling, Rule B: blob GC deferred, trigger = 20 GiB blob share or the first real deletion policy question | **Kept deferred, and deflated honestly.** §3.3 shows mark-sweep as constrained by this design reclaims only never-referenced blobs, because archival never deletes history. The real blob lever is a policy question the owner must answer (Q6), exactly as Rule B predicted. |
| Issue #4 / wave 2's bounded terminal-Work cache (`src/runtime/projection.rs:366,378`) and its miss path (`projection.rs:1141-1150`) | **Load-bearing new constraint, and a beneficiary.** The miss path is a *full* journal replay under the core guard. At 1M events that is a ~26–34 s stall on `sgt work show` (§1.5). The archive manifest's per-segment work-id set turns it into a two-segment read — ~35× less time under the guard (§4.5). |
| Issue #159 / estate-root contract §12.1, §12.3: durable artifacts are never auto-deleted; explicit deletion is a separate maintenance action | **Followed exactly.** `sgt journal archive` previews by default, acts only on `--yes`, and deletes nothing at all. |
| `sgt work reap --yes` (`src/cli.rs:422`, `POST /v1/work/{id}/reap`) | **Precedent adopted** for the verb's shape: explicit, confirmed, reported per item, never scheduled. |
| ADR 0008 (manifest authority over storage paths) | **Respected.** The archive lives inside the already-resolved data dir (`<data-dir>/journal/archive/`). No new manifest key, no new resolution rung. |
| ADR 0012 (estate and doctor are daemon API surface) | **Binding.** The journal holds an exclusive advisory lock for the daemon's lifetime (`src/runtime/journal.rs:68,256`), so the archive verb cannot be an offline file mover. It is a daemon API call (§4.7). |
| `src/runtime/recovery.rs:118-123` — stranded `surfaces/<work-id>/` directories, "a garbage collector's job… Tracked as issue #17" | **Named, and left out of scope** (§7). Surface-directory reclamation shares this issue's number but not its invariants; it is a filesystem sweep against journaled evidence, and belongs with #159's verb, not with the replay contract. |
| Kickoff ruling 7 (2026-08-20): every default argued for Linux/macOS/WSL, never tuned to Cerberus | **Binding on every number in §4.** Cerberus's 54.6k ev/s appears once, as one point in a platform spread, and is never the basis of a default. |

────────

## 1. Problem statement

### 1.1 What is unbounded

Four stores grow monotonically and none of them has a policy:

| Store | Path | Growth mechanism | Bound today |
|---|---|---|---|
| Journal segments | `<data-dir>/journal/00000001.ndjson` … | Rotates at `DEFAULT_SEGMENT_MAX_BYTES = 8 MiB` (`src/runtime/journal.rs:52`); a new segment is created, none is ever retired | None |
| Blob store | `<data-dir>/blobs/b3/<64-hex>` | Every Claude turn's raw stream-json (`src/backend/claude.rs:1345-1349`) and every Docker execute stage's stdout/stderr (`src/backend/docker.rs:817-828`) is written write-once | None; `blob.rs` has no delete API and no enumeration API at all |
| DuckDB | `<data-dir>/projections/sergeant.duckdb` | Deleted and rebuilt wholesale from the journal on every start (`src/runtime/analytics.rs:708`) | Bounded *by the journal* — it has no growth of its own, only the journal's |
| In-memory projection | — | Was unbounded (~25 kB/work); **now bounded** by wave 2's caches at 512 runs / 1024 works | Bounded, except the slim index, which is one row per Work ever journaled and never evicted (`projection.rs:410-440`) |

The load-bearing observation is that only the first two are *independently*
unbounded. DuckDB is derived, so it inherits the journal's horizon and needs
no policy of its own. The in-memory projection was fixed in wave 2. **Rule C
is therefore two problems, not four: journal segments and blobs.**

### 1.2 Measured costs (P1-PERF baseline, `docs/perf/baseline-2026-08-10.md`)

| Quantity | Measured value | Where |
|---|---|---|
| Journal bytes per event | 549 B steady (547 B in S2) | S2, S5 |
| Events per trivial 2-stage work | exactly 16 | S1, S2 |
| Journal floor per trivial work | ~8.8 kB | derived |
| Segment rotation | ~8.4 MB (8 MiB exact) | S5 |
| DuckDB on disk | 31.8 MiB at 50k events = **667 B/event** | S5 |
| Cold start, 50k events | 1.71 s / 178 MB RSS; 29.2k ev/s raw | S5 |
| Cold start, fixed overhead | ~425–450 ms, load-independent | S5 |
| Journal on disk, 50k events | 27.4 MB | S5 |
| Deepest single work measured | 1,010 events | S3 |
| Blob bytes | **never measured** | — |

Two facts about that table matter more than the numbers in it.

**First: DuckDB costs about as much disk as the journal it is derived
from.** 667 B/event against 549 B/event. The derived store is not a rounding
error beside truth; it is 1.2× truth. (The S2 figure of 2,621 B/work and the
S5 figure of 31.8 MiB at 50k events — 10.7 kB/work at 16 events/work —
disagree by ~4×. Per-event is the reconcilable unit; the per-work spread is
itself a measurement item, §5.2.)

**Second: every number above was produced against the fake backend
(`§37`).** The fake backend never produces a Claude raw transcript. So the
one term most likely to dominate real disk consumption — blob bytes — has
**no measurement at all**. Any growth projection that omits blobs is a
projection of the smallest term.

### 1.3 When this actually bites — the arithmetic

**The binding constraint is rebuild time, not bytes.** That was the owner's
2026-08-11 amendment and the measurements support it: the trigger is a cold
start exceeding 30 s, and a cold start happens on every upgrade, every
crash, every machine reboot.

Extrapolating the measured rate to 1M events:

```
raw rate      1,000,000 / 29,200 ev/s              = 34.2 s
marginal rate 0.44 s + 1,000,000 / 39,400 ev/s     = 25.8 s
```

(The marginal rate removes the measured ~440 ms fixed overhead:
50,000 / (1.71 − 0.44) = 39.4k ev/s.) Cerberus measured 54.6k ev/s, which
puts the same mark at 18.8 s. So across one known platform spread, **1M
events is 19–34 s of cold start — at or past the ruled 30 s trigger, not a
safe distance from it.** The 2026-08-11 ruling estimated the trigger at
~1.6M events from Cerberus's rate alone; argued across the platform target
set (ADR 0001: Linux, macOS, WSL) rather than from the fastest measured
host, the trigger arrives at roughly 1M.

Disk at that same mark:

```
journal   1,000,000 × 549 B = 549 MB   (65 segments)
DuckDB    1,000,000 × 667 B = 667 MB
                              ------
                              1.22 GB, plus blobs
```

**When is 1M events reached?** There is no measured event count for a real
actor-driven Work. The only two anchors are 16 events (trivial, fake
backend) and 1,010 events (S3's deliberately deep single work). The table is
therefore parametric between them:

| events/work | works to 1M | at 10 works/day | at 50 works/day | at 300 works/day |
|---|---|---|---|---|
| 16 (measured floor) | 62,500 | 17 years | 3.4 years | **208 days** |
| 100 | 10,000 | 2.7 years | 200 days | **33 days** |
| 500 | 2,000 | 200 days | 40 days | **6.7 days** |

The issue's own framing — "hundreds of works/day… months, not years" — is
correct in the middle of that band and **optimistic at the top of it**. At
the issue's stated heavy-use rate and a plausible real event density, the
trigger is reached in weeks.

**Blobs, illustrated (not measured).** A single Claude turn's raw
stream-json is the largest artifact the system writes per event. At an
illustrative 50 kB/turn and 40 turns/work that is 2 MB/work — **227× the
journal floor**. At 300 works/day that is ~600 MB/day, and the 20 GiB
blob-share backstop from Rule B arrives in about five weeks. Every input to
that sentence is unmeasured, which is precisely why blob measurement is item
1 of §5.

### 1.4 Why the existing visibility does not catch it

`disk_pressure_check` (`src/cli.rs:3018`) fails below 100 MiB free and warns
below 1 GiB free (`cli.rs:3042-3043`). Those thresholds are about the
*filesystem's* headroom. On a 1 TB disk with 600 GB free, the data dir can
grow to 599 GB while the check reports `ok`. The check measures the wrong
axis for this problem: it sees headroom, never growth, and it never sees
rebuild time at all.

### 1.5 The constraint that did not exist when Rule C was deferred

Wave 2 landed the bounded terminal-Work cache for #4. Its miss path is:

```rust
fn rederive_registry_for(journal: &Journal, work_id: &str) -> Result<WorkRegistry, JournalError> {
    let mut scratch = WorkRegistry::default();
    for event in journal.replay()? {
        let event = event?;
        if event.work_id.as_deref() == Some(work_id) {
            apply_registry_event(&mut scratch, &event, false);
        }
    }
    Ok(scratch)
}
```
`src/runtime/projection.rs:1141-1150`

This is a **full** journal walk from seq 1 with an application-level filter —
`journal.replay()`, not `replay_after` — run under `blocking_sync` while the
core guard is held (`src/api.rs:1514,1569`). At 1M events, `sgt work show`
on the 1,025th-oldest terminal Work costs 26–34 s **and queues every other
request behind it**. That is not a slow read; it is a daemon-wide stall.

So Rule C now has a second consumer of "full replay" to answer for, and it
is the more user-visible one. Any archival design that makes this path
worse is disqualified. §4.5 shows the recommended design makes it ~35×
better.

### 1.6 The startup cost nobody has named

A cold start replays the journal **three times**:

1. `Journal::open` recomputes `next_seq` by replaying every event
   (`src/runtime/journal.rs:265-268`) — the only thing it keeps is the last
   `seq`.
2. `registry.catch_up(journal.replay()?)` — the work projection fold
   (`src/daemon.rs:460-461`).
3. `Analytics::rebuild(data_dir, journal.replay()?)` — the DuckDB fold
   (`src/daemon.rs:467`).

Pass 1 is pure waste: the same answer is available from the last line of the
last segment. Passes 2 and 3 are two independent folds over the same event
stream and can share one parse. This is named here because **it is the
cheapest available lever on the binding constraint and it touches no
invariant at all** (§4.1).

────────

## 2. Invariants that bound any design

These are the fences. A design that crosses one is rejected in §3 by
reference to the number below.

### 2.1 I1 — The journal is the only truth

Every state change is a journaled event; everything else is a disposable
projection rebuilt from it (`docs/DEVELOPMENT.md:37`). No design may create
a second store of state that is not derivable from journaled events. A
derived artifact is permitted only if (a) it is reconstructible from the
journal alone, and (b) its binding to the journal state that produced it is
mechanically checkable.

### 2.2 I2 — Replay must reconstruct every Work

Two live consumers depend on this, not one:

- **cold-start rebuild** — `src/daemon.rs:460-461`, `467`;
- **the #4 miss path** — `src/runtime/projection.rs:1129,1141,1162`, reached
  from `src/api.rs:1514,1569`.

Archived data must remain reachable by both, or the archival must be an
explicitly-declared, honestly-reported hole (I5).

### 2.3 I3 — Seq continuity is the replay contract's enforcement mechanism

`Replay` checks `event.seq == self.expected` for every event, starting at 1,
and fails closed with `JournalError::SeqDiscontinuity` on a gap or a
regression (`src/runtime/journal.rs:592-675`). This is not incidental: it is
how the journal proves nothing was lost.

**Consequence: removing segment `00000001.ndjson` from
`<data-dir>/journal/` breaks replay outright today.** Any archival design
must carry an explicit `archived_through_seq` floor and teach `Replay` to
begin expecting at that floor + 1 — and must make the floor itself part of
the checkable binding, so a missing archive is a named failure and never a
silently short history.

The related mechanism already exists and is the model to copy:
`Journal::replay_after` (`journal.rs:454`) already skips whole segments
using `first_seq` (`journal.rs:574`) and resumes seq validation from the
first kept segment. Archival's floor is the same idea made durable.

### 2.4 I4 — Nothing is deleted except by an explicit maintenance action

The estate contract's rule ("Deletion is a separate explicit maintenance
action with its own authorization and dry-run evidence",
`docs/proposals/estate-root-git.md:971-973`) and the shipped precedent
(`sgt work reap --yes`) bind here. No automatic reaper, no scheduled sweep,
no on-start compaction, no deletion as a side effect of any other verb.

### 2.5 I5 — Archived data is replayable, or it is explicitly out of contract

There is no third state. Data that is on disk but not reachable by replay,
and not declared as such, is the failure mode this proposal exists to
prevent. If a rung ever moves data out of the replay contract, it must
name the consequence at the point a user meets it: `sgt work show` on such
a Work reports a declared hole, never a partial view rendered as complete.

### 2.6 I6 — ADR 0003 promises resumability, not never-delete

This needs saying plainly because the issue's framing assumes otherwise.
ADR 0003 nowhere promises that data is retained forever. What it promises is
that "the journal makes work **resumable**" and that "everything but the
journal is a disposable projection rebuilt from it"
(`docs/adr/0003-durability-promise-and-storage-preconditions.md:24-27`).

Therefore:

- **Archival that keeps history replayable requires no amendment to ADR
  0003.** The recommended composition (§4) does not amend it.
- **Any rung that discards history amends ADR 0003 and must say so in its
  own ADR**, naming the consequence: a Work whose evidence was discarded can
  no longer be reconstructed, and the durability promise for that Work has
  been traded away by an explicit operator act.

This is pre-committed here as a tripwire so that a later rung — blob
evidence deletion (Q6), or work-retirement compaction (§3.2) — cannot cross
it as an implementation detail.

### 2.7 I7 — Defaults are argued for the platform targets

Kickoff ruling 7. Every default in §4 is derived from a stated budget and
the platform target set (Linux, macOS, WSL — ADR 0001), never from the
fastest measured host. Cerberus is a measurement point, not the design
target.

### 2.8 I8 — Single writer; the archive verb is daemon API surface

The journal holds an exclusive advisory lock at
`<data-dir>/journal/.lock` for the handle's lifetime
(`src/runtime/journal.rs:68,256`), and the daemon holds that handle for as
long as it runs. An offline file mover would either fail to take the lock or
race a live writer. Combined with ADR 0012 (estate and doctor are daemon API
surface), the archive verb **must** be a daemon API call with a CLI front
end, exactly like reap.

────────

## 3. The design space

Four options, each with mechanism, invariant impact, and disposition.

### 3.1 (a) Age/size/count-triggered segment archival

**Mechanism.** Cold segments move to `<data-dir>/journal/archive/`, a
manifest records which seq ranges live there, and replay reads archived
segments only on demand.

**Invariant impact.** I3 is the whole difficulty. Two variants exist and
they are not the same design:

- **(a-i) Archive without skipping replay.** Cold start still reads every
  archived segment, just from a different directory (and, if compressed,
  through a decoder). I1–I5 all hold trivially. **But it saves no rebuild
  time** — it saves disk, which is the axis the trigger is not on.
- **(a-ii) Archive and skip replay.** Cold start reads only the live
  window. This is what actually attacks the binding constraint. It also
  immediately violates I2: the works whose events live only in archived
  segments would vanish from `work list`, because the slim index
  (`projection.rs:410-440`) is itself rebuilt by full replay and has no
  durable form.

**This is the crux of Rule C, and the 2026-08-11 ruling identified it
exactly:** "Segments interleave works, so archiving 'old' segments while
rebuild-on-start replays the full journal is a contradiction — segment
retirement requires a load-bearing snapshot/checkpoint, which is exactly the
B1 machinery ruled dormant."

The ruling is right that (a-ii) requires persisting something derived. What
this proposal adds is a measurement of *how much*: the durable artifact
needed is not a projection snapshot. It is `WorkIndexRow` — six scalar
fields (`id`, `intent`, `state`, `integrity`, `created_at`, `updated_at`),
plus one added `last_seq` (§4.3), all of which are **terminal-immutable for
exactly the Works that qualify for archival**. At ~200 B/row that is 12.5 MB
for 62,500 Works. And unlike a
projection snapshot, it is re-derivable at any moment by replaying the
archive, which makes its binding checkable (I1b) rather than trusted.

**Disposition: ADOPTED**, as (a-ii) with the narrowed durable artifact, and
gated on the owner permitting that narrowing (Q2). If Q2 is ruled against,
this collapses to (a-i) and Rule C can only ever address disk.

### 3.2 (b) Work-retirement-based compaction

**Mechanism.** When a Work retires, rewrite its trajectory into a single
compound snapshot event and drop the originals.

**Invariant impact.** Three independent violations:

1. **I3.** The journal is append-only with a validated contiguous seq
   chain. Replacing N events with 1 either leaves a hole (fails
   `SeqDiscontinuity`) or requires renumbering every subsequent event —
   which invalidates every `causation_id`, every DuckDB primary key (the
   schema is keyed on journal seq, `src/runtime/analytics.rs:110-234`),
   and every external reference to a seq.
2. **I6.** The original trajectory is destroyed. This *is* discarding
   history, and it discards it automatically at retirement rather than by
   explicit operator act — violating I4 as well.
3. **Unverifiability.** Once the originals are gone, the compound event
   cannot be checked against what it claims to summarize. The compaction
   becomes new truth, not derived truth (I1).

**Interaction with B1, named as required.** This is B1's dormant snapshot
machinery at full strength — not the narrowed index-row form of §3.1, but
identity binding over arbitrary reconstructed Work state, which is the exact
reason B1 was ruled dormant.

**Disposition: REJECTED** on all three counts. It is recorded here rather
than dropped because it is the only option that would shrink cold start
*below* the floor archival can reach (the live window still has to be
replayed). If the measurements in §5 show archival cannot hold the line, this
is the rung to reach for — with an ADR amending 0003, per I6.

### 3.3 (c) Blob GC via liveness

**Mechanism.** Determine which blobs are live and delete the rest.

**What makes a blob live?** Under this design: *referenced by any event in
the live journal or in any archived segment.* Because archival never deletes
a segment, **archival never changes blob liveness.**

**Refcounting.** Requires a durable counter, mutated on every event append
and every archival act, crash-safe across both. That counter is state that
is not derivable from the journal without recomputing it — I1. It also puts
a second durable write on the append hot path, against the group-commit
design (`journal.rs:343-355`). Cost: O(1) per append, O(1) per delete, plus
a crash-recovery story for the counter itself. **Rejected on I1.**

**Mark-sweep.** One full journal replay marks every referenced blob; one
directory listing sweeps the rest. Cost: one full replay (26–34 s at 1M
events) plus one listing of a **flat** directory
(`<data-dir>/blobs/b3/<hex>`, `src/runtime/blob.rs:92` — no prefix
sharding), which at 100k+ blobs is itself slow on every target filesystem.
Derivable from truth, so I1 holds.

**A mechanical hazard that must be stated.** Blob references are *not* a
typed field. `Event` has no blob field (`src/domain/event.rs:45-82`); refs
live only as `"b3:<hex>"` strings inside the untyped `payload: Value`, under
ad hoc keys chosen per producer — `"raw"` (`src/backend/claude.rs:1399,
1419,1551`), `"stdout"` and `"stderr"` (`src/backend/docker.rs:1012,1015`).
A typed mark pass is therefore impossible. A correct mark must walk the
entire payload JSON for any string matching `b3:<64 hex>`. That is
conservative in the safe direction (over-marking never deletes a live blob),
but it means **any future payload that stores a ref in a non-string form
becomes a silent deletion bug.**

**The honest deflation.** Given the liveness definition above and the fact
that archival retains every segment, mark-sweep reclaims exactly one class
of blob: those written but never referenced by any event — the crash window
between `BlobStore::put` and the journal append that names them. That is a
real class and it is small. **Blob GC as constrained here delivers almost
nothing.** The real levers on blob disk are (i) explicit per-Work evidence
deletion, which is a *policy* question about what "retire" means (Q6), and
(ii) bounding or compressing what is written into blobs in the first place.

**Disposition: mark-sweep ADOPTED IN PRINCIPLE, DEFERRED IN PRACTICE.**
Rule B's existing trigger stands (20 GiB blob share, or the first real
deletion-policy request). This proposal adds two acceptance obligations so
the eventual sweep is cheap: the archive manifest already carries per-segment
work-id sets, and no code may write a blob without a journal event naming it
(Rule B's own acceptance clause, restated).

### 3.4 (d) Do-nothing-with-monitoring

**Mechanism.** Ship no retention. Extend Rule B's visibility so the horizon
is measured rather than assumed.

**Invariant impact.** None. This is the R1 rung and it is genuinely
defensible — for a while.

**But it cannot be done by raising the existing thresholds.** The shipped
check is free-space-based (`cli.rs:3042-3043`), and raising `WARN_BELOW`
from 1 GiB to, say, 20 GiB free would fire on every small laptop while still
never firing on a large disk filling up. The axis is wrong. What "monitoring"
has to mean here is a *growth and rebuild-time* axis: live journal bytes and
event count, archived bytes, blob share, and the measured duration of the
last cold start.

**When it stops being defensible.** Stated as a mechanical tripwire, not a
feeling:

1. any real estate's measured cold-start rebuild exceeds **10 s** (a third
   of the ruled 30 s trigger — the point at which the trigger is one
   doubling away rather than an abstraction); or
2. any real estate's blob share crosses Rule B's 20 GiB backstop; or
3. the #4 miss path (§1.5) exceeds **2 s** on a real estate, because that
   stall is taken under the core guard and is felt by every concurrent
   request.

Until one of those fires, do-nothing is the correct engineering answer. The
problem is that today **none of the three is measurable**, so "do nothing"
is currently indistinguishable from "don't know."

**Disposition: ADOPTED as rung 1** of the composition — not as the whole
answer, but as the thing that must ship first and must ship regardless of
how the rest is ruled.

### 3.5 A correction offered on the pre-committed ladder

The 2026-08-11 adjudication pre-committed the ladder as: (1) compress cold
segments in place, and only if that fails, (2) snapshot/truncate.

Compression is a good idea and §4 adopts it. But the arithmetic says it does
not address the trigger it was pre-committed against:

- The binding trigger is **rebuild time**.
- Replay measures ~21.6 MB/s of JSON (27.4 MB in ~1.27 s of marginal time).
  A modern NVMe reads that in ~7 ms. Replay is not I/O-bound; it is
  parse-bound. Compression reduces bytes read and adds decode CPU. The net
  effect on rebuild time is somewhere between "slightly positive" and
  "neutral" — it is not a 10× lever on time the way it is on disk.
- Compression is, however, a genuine ~5–10× lever on the *disk* axis, which
  is the axis Rule B's blob backstop is on.

So: keep compression, apply it where it works, and do not expect it to hold
the 30 s line. The rung that actually moves rebuild time and costs no
invariant at all is the one nobody has named — collapsing three startup
replays into one (§1.6, §4.1). This is offered as Q1 rather than assumed.

────────

## 4. Recommended composition

Five rungs, cheapest-first. Each is separately shippable and each is
useless-but-harmless if the one above it is skipped.

### 4.1 Rung 0 — one cold-start replay instead of three

**No invariant impact whatsoever.** Pure implementation.

- `Journal::open` stops replaying the whole journal to find `next_seq`
  (`journal.rs:265-268`) and reads the last complete line of the last
  segment instead. `recover_tail` already runs first, so that line is
  guaranteed well-formed. Fail closed if it cannot be read, exactly as
  today.
- The registry fold and the DuckDB fold share one `Replay` iterator, applying
  both folds per parsed event, instead of two independent full walks
  (`daemon.rs:460-461`, `467`).

**Expected effect:** up to ~3× on the parse-bound portion of cold start —
i.e. the 30 s trigger moves from ~1M events to ~3M. The true share of
startup that is parse-bound is **unmeasured**, which is why this is item 3
of §5 and not a claim.

Ship this first. If it delivers, every later rung gets more headroom and the
1M-event measurement is taken against the improved baseline.

### 4.2 Rung 1 — Rule B gains a growth axis

A new `sgt doctor` check, `journal_growth`, beside `disk_pressure` rather
than replacing it (`cli.rs:2284`). Journal-and-filesystem derived, no git
walk, and it must not add to doctor's measured ~450 ms floor.

```text
journal growth
  live segments:              57 (456 MiB, ~871k events)   [warn: > 2× window]
  archived segments:          none
  replay floor:               seq 1
  blob store:                 4.2 GiB (61% of a 6.9 GiB data dir)
  last cold-start rebuild:    22.1 s (871k events, 39.4k ev/s)   [warn: ≥ 10 s]
  slowest work re-derive:     19.4 s
```

Thresholds, argued from §3.4's tripwires and nothing else:

| Signal | warn | fail |
|---|---|---|
| last cold-start rebuild | ≥ 10 s | ≥ 30 s (the owner-ruled trigger, made mechanical) |
| blob share of data dir | ≥ 20 GiB | — (Rule B's backstop; fires a design contract, not an alarm) |
| live journal | ≥ 2× the archive window (§4.3) | — |

The rebuild duration and the event count are recorded on `daemon.started`
so doctor reads them from the journal rather than measuring anything itself.
The re-derive figure is the slowest #4 miss-path replay observed since
start, held in memory (it is diagnostics, not state).

The remedy line names `sgt journal archive --dry-run`. It deletes nothing.

**This rung ships regardless of how Q1–Q7 are ruled.** It is what turns
"do nothing" from a guess into a measurement.

### 4.3 Rung 2 — cold segment archival with a manifest

**Layout.**

```text
<data-dir>/journal/
  00000042.ndjson                  live segments (unchanged)
  00000043.ndjson
  .lock
  archive/
    manifest.json
    00000001.ndjson.zst
    00000002.ndjson.zst
    ...
```

Inside the already-resolved data dir; no new manifest key, no new resolution
rung (ADR 0008).

**`manifest.json`** — itself a derived artifact, re-buildable by reading the
archived segments:

```json
{
  "schema": "sergeant.journal-archive/v1",
  "archived_through_seq": 580678,
  "segments": [
    {
      "index": 1,
      "first_seq": 1,
      "last_seq": 15281,
      "events": 15281,
      "raw_bytes": 8388401,
      "stored_bytes": 998112,
      "codec": "zstd",
      "blake3": "b3:…",
      "work_ids": ["01H…", "01H…"]
    }
  ],
  "index_rows": [
    { "id": "01H…", "intent": "…", "state": "completed",
      "integrity": "clean", "created_at": "…", "updated_at": "…",
      "last_seq": 15044 }
  ]
}
```

`last_seq` is not part of `WorkIndexRow` today; it is added because it is
what lets the miss path skip the live window entirely (§4.5). It costs one
`u64` per row and is derived from the same pass that evaluates eligibility.

**Archival eligibility.** A segment is archivable if and only if:

1. it is not the currently-open segment; **and**
2. it is older than the live window (§4.4); **and**
3. every event in it carries a `work_id` whose Work is **terminal and
   settled** — the same `is_absorbing` + `run_is_settled` predicate wave 2
   already uses for cache eviction (`projection.rs:644,701`) — and whose
   `WorkIndexRow` is written into `index_rows`; **and**
4. every event in it *without* a `work_id` is of a kind on an explicit
   archivable allowlist (Q5).

Condition 3 is the ruling's "segments interleave works" objection answered
directly rather than argued around: a segment holding even one event of a
live Work is simply not eligible. **Named consequence: one long-lived Work
can stall archival indefinitely.** That is accepted, not worked around — the
`journal_growth` check reports the blocking Work by id so the operator sees
*why* the live journal is not shrinking, and the honest remedy is to finish
or cancel that Work.

**Replay contract change.** `Replay` gains a floor. Given a manifest with
`archived_through_seq = N`:

- **Live replay** (cold start, `Journal::replay`) begins expecting seq
  `N+1`, and validates that the first live segment's `first_seq` is exactly
  `N+1`. A mismatch is a hard failure with a named remedy, never a shortened
  history.
- **Full replay** (`Journal::replay_from_archive`, new) reads archived
  segments in index order, verifying each against its recorded `blake3`
  before decoding, then continues into the live segments — one continuous
  seq chain from 1, exactly as today.
- `replay_data_dir` (doctor's lock-free path, `journal.rs:473`) reads the
  manifest and reports the floor, so an archived history never reads as a
  seq gap. Its torn-tail classification (`journal.rs:87-101,166-175`) is
  unaffected: archived segments are immutable and closed, so a torn tail is
  still only ever possible on the last line of the last live segment.

**Compression** rides the move: a segment is compressed as it is archived
(`.ndjson.zst`), never in place, never on a live segment. This is the
2026-08-11 ladder's rung 1, folded in at zero extra machinery. Expected
5–10× on NDJSON with repetitive keys — **unmeasured**, §5.4.

**Nothing is deleted.** Archival is a move plus a compress. The bytes stay.

### 4.4 The live-window default, argued from platform posture

The window is the amount of journal a cold start must always replay. The
budget it is derived from:

> A cold start should complete in under 5 s on the slowest platform target,
> with the ruled 30 s trigger as a never-exceed.

Working:

- Measured marginal replay rate on the P1-PERF container class: ~39.4k ev/s
  (§1.3). Cerberus: 54.6k ev/s.
- The platform target set is Linux, macOS, and WSL (ADR 0001). Neither a
  WSL2 distro nor a macOS host has ever been measured for replay. Pending
  measurement, assume a **2× penalty** against the baseline container class
  for the slowest member — a deliberately pessimistic placeholder, replaced
  by measurement in §5.5. Floor: **~20k ev/s**.
- 5 s × 20k ev/s = 100,000 events ≈ 55 MB ≈ 6.5 segments.
- Round up, because condition 3 of §4.3 means the *effective* window is
  always larger than the nominal one (segments wait for their Works to
  settle):

> **Default live window: the newest 16 segments, or 128 MiB of live
> journal, whichever is larger.**

(16 × 8 MiB = 128 MiB at the default rotation threshold; the "whichever is
larger" clause keeps the rule sane if `segment_max_bytes` is ever
overridden.)

Sanity across the platform spread — 128 MiB ≈ 245k events:

| rate | cold start on a full window |
|---|---|
| 20k ev/s (pessimistic floor) | 12.2 s |
| 39.4k ev/s (measured baseline class) | 6.3 s |
| 54.6k ev/s (Cerberus) | 4.5 s |

Inside the 30 s trigger with ≥2.5× margin on the worst assumption, and
inside 10 s on every rate that has actually been measured. If rung 0 lands
first, every figure improves by up to ~3×.

**No configuration surface.** Per the 2026-08-11 ruling's own "what this
deliberately does not do" — policy knobs without a mechanism to obey them
are dead weight. The window is a constant, revisable by evidence.

**When to run the verb** (not a trigger to act automatically — I4): doctor
recommends it when the live journal exceeds 2× the window (32 segments /
256 MiB), the point at which one pass reclaims at least half the live
journal.

### 4.5 Rung 3 — archival repairs the #4 miss path

This is the argument that makes rung 2 worth its complexity even before the
rebuild-time benefit lands.

`rederive_registry_for` today walks every event and discards ~99.99% of them
(§1.5). With the manifest, the miss path becomes:

1. `work_index` confirms the id exists (already the case, `api.rs:1522`);
2. the manifest's `work_ids` sets name the archived segments containing that
   Work — typically one or two, because a Work's events are contiguous in
   time;
3. the index row's `last_seq` decides whether the live window can be skipped
   entirely: if `last_seq <= archived_through_seq`, no live segment can
   contain an event for that Work, so none is read;
4. replay reads only the named segments, filtered as today.

At 1M events with a 16-segment live window, for a fully-archived Work:
**two archived segments (~30k events) instead of 1M** — roughly 0.8 s
instead of 26–34 s, a ~35× reduction in how long the core guard is held.
For a Work entirely inside the live window (the common case) the cost is
unchanged. For the boundary case — a Work spanning the archive floor —
step 3 fails and the live window is read too, giving ~7 s at that scale:
still 4× better, and it is the rarest of the three cases.

Recording `work_ids` and `last_seq` in the manifest costs nothing extra: the
archival pass already reads every event in every candidate segment to
evaluate eligibility condition 3.

### 4.6 Rung 4 — blob liveness, kept deferred

No blob deletion ships. Rule B's trigger stands unchanged. What ships is the
groundwork that makes the eventual sweep cheap and correct:

- the per-segment `work_ids` set (already required by §4.5);
- an acceptance test that **no code path writes a blob without a journal
  event naming it** — Rule B's own acceptance clause, restated because the
  mark pass's correctness rests entirely on it;
- an acceptance test that a generic `b3:<64hex>` payload walk marks a blob
  written by *every* producer path (`claude.rs` `raw`, `docker.rs` `stdout`
  and `stderr`), so the untyped-payload hazard of §3.3 is pinned by a test
  rather than by a comment.

`sgt doctor` reports blob share against Rule B's 20 GiB backstop (§4.2). If
that fires, a blob-GC design contract fires with it — as already ruled.

### 4.7 The maintenance verb

```text
sgt journal archive              # preview; the default, always safe
sgt journal archive --yes        # act
sgt journal verify               # re-hash every archived segment against the manifest
```

Both are daemon API calls with CLI front ends (`POST /v1/journal/archive`,
`GET /v1/journal/archive`), because the daemon holds the journal's exclusive
lock for its lifetime (I8, ADR 0012). Both follow `sgt work reap`'s shape:
explicit, previewed, reported per item, never scheduled, never invoked as a
side effect of another verb.

The preview reports exactly what `--yes` would do:

```text
sgt journal archive --dry-run

  eligible:        38 segments (seq 1–580,678), 304 MiB
  would archive:   38 segments → ~36 MiB compressed (estimated at 8.4×)
  would retain:    19 live segments (152 MiB, ~290k events)
                   replay floor seq 580,679; est. cold start 7.4 s
  blocked:         3 segments held by work 01HXYZ… (running, submitted 6d ago)
  deletes:         nothing — archival moves and compresses; no bytes are removed

  run with --yes to proceed
```

The archival pass itself:

1. takes the core guard (single writer, I8);
2. evaluates eligibility over candidate segments;
3. for each eligible segment: compress to a temp file, `fsync`, rename into
   `archive/`, `fsync` the directory, then `fsync`-rename the updated
   manifest, then unlink the original — **manifest before unlink**, so a
   crash at any point leaves either the original or a manifest-listed
   archive, never a hole;
4. journals `journal.archived` with the seq range, segment count, and byte
   counts, so the act is itself in the history it archives.

`sgt journal verify` re-reads every archived segment, checks its `blake3`,
and checks that the seq chain across archive-plus-live is contiguous from 1.
It is the answer to "is my history intact" and it is what the acceptance
criteria in §6 test against.

────────

## 5. Measurement plan — what to measure before implementing

The issue names one measurement ("measure rebuild at 1M events"). It needs
five. Every one runs on the committed harness (`scripts/perf/`), which
scaffolds its own scratch estate — `perf_init` creates `<outdir>/scratch`,
`perf_seed_repo` builds a throwaway git repo and workflow, and
`perf_daemon_start` runs the release binary against a scratch data dir
(`scripts/perf/common.sh`). Nothing touches an estate mount, and `<outdir>`
must live outside the repo tree.

### 5.1 Rebuild at 1M events (the issue's named gate)

`s5-journal.sh` already does exactly this and takes its marks from the
environment:

```sh
PERF_S5_MARKS="100000 250000 500000 1000000" \
PERF_S5_READY_TIMEOUT=1800 \
  scripts/perf/s5-journal.sh /var/tmp/perf-out/s5-1m
```

Records per mark: cold start ms, rebuild ev/s, RSS after start, journal
bytes, DuckDB bytes, segment count, and every canned analytics query's
latency (`s5-marks.csv`, `s5-analytics.csv`, `s5-growth.csv`).

**What it decides.** Whether 1M events is 19 s or 34 s or worse; whether the
30 s trigger is at 1M, 1.6M, or 3M events; and whether the rebuild rate
holds, improves, or degrades with scale (it improved from 14.6k to 29.2k
ev/s across 10k→50k; whether that continues is unknown). Also whether the
178 MB-at-50k RSS figure extrapolates linearly — a naive extrapolation gives
3.5 GB at 1M, which is not credible and is exactly why the mark is being
measured rather than assumed.

Note that `PERF_S5_MARKS` at 1M against the fake backend means ~62,500
works, which will take hours to generate. Budget for it.

### 5.2 Blob growth against a real actor

The gap that matters most, because it is the term with no measurement at
all. Every P1-PERF number came from the fake backend, which writes no
transcript blobs.

Measure, over a run of real Claude-backed Works: bytes per turn written to
`<data-dir>/blobs/b3/`, turns per Work, and total blob bytes per Work — and
the same for a Docker execute stage's stdout/stderr capture. Rule B's
adjudication already fixed the vehicle for the execute-stage half: the real
local Docker engine with a Sergeant-built probe image, not the fake backend.

**What it decides.** Whether §1.3's illustrative 2 MB/work is the right
order of magnitude, and therefore whether the blob backstop or the rebuild
trigger arrives first. If blobs dominate as expected, the composition's
ordering is unchanged (rungs 0–2 are still cheapest-first) but Rule B's
blob-GC trigger becomes the near-term one, and Q6 becomes urgent.

Also reconcile the DuckDB per-work discrepancy while here (2,621 B/work in
S2 vs 10.7 kB/work implied by S5) — per-event is the reconcilable unit and
667 B/event is the figure §1 uses.

### 5.3 The three-replay split (rung 0's justification)

At the 250k and 1M marks, time each startup phase separately: `Journal::open`'s
`next_seq` scan, the registry fold, the DuckDB fold, and the listener bind.

**What it decides.** Whether rung 0 is a 3× win, a 2× win, or noise. If the
DuckDB fold dominates and the two folds cannot usefully share a parse, rung
0 shrinks to "delete the `next_seq` scan," which is still free.

### 5.4 Compression ratio and decode cost on real segments

`zstd -3` (and `-9`, for comparison) over real journal segments from the 1M
run: ratio, compress time per segment, decode time per segment.

**What it decides.** Whether the 5–10× estimate holds, and — the question
§3.5 raises — whether decoding archived segments during a full replay costs
more or less than the I/O it saves. If decode is net-negative on time, the
codec becomes a per-tier choice rather than a default.

### 5.5 Replay rate on a second platform target

The live-window default (§4.4) rests on an *assumed* 2× penalty for the
slowest platform target, because neither macOS nor WSL2 has ever been
measured for replay. Run `s5-journal.sh` at reduced marks
(`PERF_S5_MARKS="50000 250000"`) on a macOS host and inside a WSL2 distro
with the data dir on the Linux filesystem (not `/mnt/c` — ADR 0003 D6 makes
that configuration a hard refusal, not a slow one).

**What it decides.** Whether 16 segments / 128 MiB is right, generous, or
too small. Per ruling 7 this is the measurement that lets the default be
argued for the platform targets rather than assumed from one host.

### 5.6 What is NOT measured first

The #4 miss-path cost (§1.5) needs no new measurement — it is a full replay
by inspection, so §5.1's rebuild figure *is* its figure, minus the DuckDB
fold. It is called out because doctor should report it (§4.2), not because
it is uncertain.

────────

## 6. Acceptance criteria for the eventual implementation

### Replay contract

- A journal with an archive replays byte-identically, event for event, to
  the same journal before archival — verified by folding both into a
  `WorkRegistry` and comparing.
- Cold start with `archived_through_seq = N` reads **zero** archived
  segments and produces a `work_index` containing every Work ever
  journaled, archived or not.
- A missing, truncated, or hash-mismatched archived segment is a named hard
  failure at the first full replay that needs it — never a short history,
  never a silent seq gap.
- A manifest whose `archived_through_seq` does not equal
  `first live segment.first_seq − 1` is a named hard failure at daemon
  start.
- `sgt journal verify` detects each of: a deleted archived segment, a
  flipped byte in one, a manifest listing a segment that is absent, and a
  segment present but unlisted.
- Torn-tail classification is unchanged: `possible_torn_tail` is still true
  only for the last line of the last **live** segment
  (`journal.rs:87-101,166-175`).
- `list_segments` (`journal.rs:682`) never returns an archived segment, and
  rotation never reuses an archived index — `MAX_SEGMENT_INDEX`
  (`journal.rs:74`) continues to bound the monotonic counter, which
  archival does not reset.

### Archival correctness

- A segment containing any event of a non-terminal or unsettled Work is
  never archived, and the blocking Work is named in the preview.
- A segment containing a non-work-scoped event whose kind is not on the
  allowlist is never archived.
- Crash injection at every point in the archival pass (after compress,
  after rename, after manifest write, before unlink) leaves a journal that
  passes `sgt journal verify` and a daemon that starts.
- The pass journals `journal.archived`; a replay of the post-archival
  journal contains that event.
- Archival deletes no bytes. A byte-count assertion before and after: raw
  archived bytes are recoverable by decompression, and no blob is touched.

### #4 miss path

- `rederive_work` on a Work whose events are entirely archived reads only
  the segments the manifest names for it, and returns a `Work` identical to
  the pre-archival re-derivation.
- The miss-path replay budget is asserted in a test at a seeded scale, so a
  regression to full-replay is caught.

### Diagnostics and verbs

- `journal_growth` adds no measurable time to doctor's ~450 ms floor,
  performs no git subprocess call, and deletes nothing.
- `sgt journal archive` without `--yes` performs no filesystem mutation —
  asserted by mtime/inode comparison over the whole journal directory.
- The verb refuses when the daemon is not running, with the remedy, rather
  than moving files behind the daemon's lock.

### Blob groundwork

- No code path writes a blob without a journal event naming it (every
  producer covered).
- A generic `b3:<64hex>` payload walk marks a blob written by every producer
  path — `claude.rs`'s `raw`, `docker.rs`'s `stdout` and `stderr`.

### Platform posture

- Every default is expressed as a named constant with a doc comment stating
  the budget it was derived from and the platform target set it was argued
  for — not the host it was measured on.

### Follow-on issues to file on ratification

**1. `[journal] Cold start replays the journal three times`**
`Journal::open` recomputes `next_seq` by replaying every event
(`src/runtime/journal.rs:265-268`) purely to read the last seq, and the
registry fold (`daemon.rs:460-461`) and DuckDB fold (`daemon.rs:467`) then
walk the same stream independently. Read `next_seq` from the last complete
line of the last segment instead, and share one parse between the two folds.
This is the cheapest available lever on the ruled rebuild-time trigger, it
touches no invariant, and it should land before any archival work so later
measurements are taken against the improved baseline.

**2. `[doctor] journal_growth: growth and rebuild-time visibility (Rule B's missing axis)`**
`disk_pressure` measures free space, so a data dir growing to 500 GB on a
disk with 600 GB free reports `ok` forever. Add a `journal_growth` check
reporting live/archived segment counts and bytes, event count, replay floor,
blob share, and the last cold-start rebuild duration (recorded on
`daemon.started`), warning at 10 s rebuild and failing at the owner-ruled
30 s. This ships regardless of how the archival design is ruled — it is what
turns "no retention needed yet" from an assumption into a measurement.

**3. `[perf] Measure rebuild, RSS, blob growth, and startup phase split at 1M events`**
Run `s5-journal.sh` with `PERF_S5_MARKS="100000 250000 500000 1000000"`,
timing each startup phase separately, and separately measure real blob
growth per turn and per Work against a real Claude backend and a real Docker
execute stage — the term every existing baseline omits, because P1-PERF ran
the fake backend throughout. Add a reduced-mark run on macOS and inside WSL2
to replace the assumed 2× slow-platform penalty with a measurement. This is
the gate the 2026-08-11 ruling set on Rule C and it blocks issue 4.

**4. `[journal] Implement segment archival: archive/ + manifest + replay floor + sgt journal archive`**
Implement §4.3–§4.7 of `docs/proposals/journal-archival-rule-c.md`: move
cold, fully-settled segments to `<data-dir>/journal/archive/` compressed,
record ranges/hashes/work-ids/index-rows in a manifest, teach `Replay` a
non-zero seq floor, and add the explicit preview-by-default `sgt journal
archive` and `sgt journal verify` daemon verbs. Nothing is deleted; archived
data stays inside the replay contract. Blocked on issue 3's measurements and
on the owner's ruling of Q2 (whether the slim index may be persisted).

────────

## 7. Explicit non-goals

This proposal does not:

- delete any journal event, any segment, or any blob, automatically or
  otherwise;
- add a retention configuration surface to `sergeant.toml`;
- add any age-based or clock-based rule (the #4 ruling's "count-bound, no
  clocks" posture carries here);
- rewrite, renumber, or compact history (§3.2);
- introduce projection snapshotting in the general B1 sense — only the
  narrowed, re-derivable index rows of §4.3, and only if Q2 permits;
- give DuckDB a retention policy of its own (it is deleted and rebuilt on
  every start, `analytics.rs:708`; it inherits the journal's horizon);
- reclaim stranded `surfaces/<work-id>/` directories
  (`src/runtime/recovery.rs:118-123`) — same issue number, different
  invariants; that is a filesystem sweep against journaled evidence and
  belongs with #159's verb;
- shard the flat blob directory (`blobs/b3/<hex>`), which is a real
  scaling concern at 100k+ blobs and deserves its own issue;
- change the group-commit fsync design, the rotation threshold, or the
  torn-tail contract.

────────

## 8. Open questions requiring owner ruling

**Q1. Does the 2026-08-11 compress-first ladder stand as ruled?**
The ladder pre-committed compression as rung 1 against a rebuild-time
trigger, but §3.5's arithmetic says replay is parse-bound, not I/O-bound —
compression is a 5–10× lever on disk and roughly neutral on time.
*Recommendation: amend.* Keep compression, but as a rider on archival rather
than a standalone rung, and insert rung 0 (one startup replay instead of
three, §4.1) below both — it is free, invariant-neutral, and is the only
available ~3× on the axis the trigger is actually on.

**Q2. May the slim index be persisted, or does B1 stay fully closed?**
This is the load-bearing question. Archival that skips replay of archived
segments *requires* a durable form of `WorkIndexRow` — otherwise archived
Works vanish from `work list` (§3.1). If B1 stays fully closed, archival can
only ever save disk, never rebuild time, and Rule C cannot address its own
binding trigger.
*Recommendation: permit, narrowly.* Seven scalar fields per Work, terminal-
immutable for exactly the Works that qualify, ~12.5 MB at 62,500 Works,
re-derivable at will by replaying the archive, and binding-checked by
`archived_through_seq` plus per-segment BLAKE3. That is a cache with a
mechanical proof of freshness, not a second truth — which is the specific
property B1's identity binding was dormant for want of.

**Q3. Does this amend ADR 0003?**
*Recommendation: no, as specified.* ADR 0003 promises resumability, not
never-delete (§2.6), and nothing here deletes. But the tripwire should be
ratified explicitly: **any future rung that discards history amends ADR 0003
and requires its own ADR naming the consequence.** Ruling this now prevents
a later rung from crossing it as an implementation detail.

**Q4. Is the live window a fixed size, or a per-host computed budget?**
A daemon could measure its own replay rate and size the window to hit a
seconds budget on the host it is actually on.
*Recommendation: fixed size* — 16 segments / 128 MiB. Ruling 7 says defaults
are argued for the platform targets, not tuned to a host, and a self-tuning
window would make two estates on different machines behave differently for
reasons nobody can see. Report the measured rebuild time in doctor (§4.2)
so the constant can be re-argued from evidence instead of adapted silently.

**Q5. What do non-work-scoped events do to archival eligibility?**
Events with no `work_id` — `daemon.started`, estate-level events, whatever
arrives later — are not gated by any Work's terminality, and some of them
may carry registry state that a floor-started replay would then miss.
*Recommendation: conservative, allowlist-based.* Start with an **empty**
allowlist, so any non-work-scoped event blocks its segment, and add kinds to
it one at a time, each with a test proving a floor-started replay reaches
the same registry state as a full one. Slower to reclaim, impossible to get
silently wrong.

**Q6. May a Work's blob evidence be deleted by explicit operator action?**
Rule B named this as a *policy* question engineering must not answer alone,
and §3.3 shows it is the only lever that meaningfully reduces blob disk,
since archival never changes blob liveness.
*Recommendation: yes, opt-in per Work, through the existing reap surface,
and journaled.* A `blob.evidence_discarded` event records what was removed
and why, so replay reconstructs a Work that honestly reports a declared hole
in its evidence rather than failing or rendering a partial view as complete
(I5). This crosses Q3's tripwire and therefore requires its own ADR amending
0003 — which is the point of asking now rather than discovering it later.

**Q7. Is archival stalling behind one long-lived Work acceptable?**
Eligibility condition 3 means a single Work running for weeks pins every
segment it touched, and everything after them.
*Recommendation: accept it, and make it visible.* No override, no force
flag — the predicate is what keeps I2 true. `journal_growth` names the
blocking Work and its age, and the honest remedy is to finish or cancel that
Work. If measurement later shows this stalls archival in practice, the
answer is a narrower predicate (per-Work event extraction), not a flag that
waives the invariant.

────────

## 9. Final statement

After this proposal lands, the growth story is:

```text
cold start replays one window, not the whole history
cold segments move to archive/ compressed, and stay replayable
the replay floor is explicit, hash-checked, and verifiable
an evicted Work re-derives from two segments, not from all of them
sgt doctor reports growth and rebuild time, not just free space
sgt journal archive previews by default and deletes nothing
nothing is ever removed except by an operator who typed --yes
```

And the invariant survives intact:

> **The journal is still the only truth. Archival changes where the truth
> is stored and how much of it a cold start must read — never whether it
> can still be read.**
