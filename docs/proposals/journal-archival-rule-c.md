# Rule C — Journal Segment Archival, Blob Liveness, and Derived-Store Growth

> **Disposition (2026-08-21): superseded.** Issue #17's Q1–Q10 rulings
> record (2026-08-21 grilling session) and `docs/adr/0003-durability-promise-and-storage-preconditions.md`'s
> D7 amendment + `docs/adr/0019-bounded-retention.md` supersede this
> proposal's own recommendations wherever they conflict — most importantly,
> the compressed-archive design below (§§3–6: the `archive/` tier,
> per-segment manifests, the `sgt journal archive` preview verb) was
> replaced wholesale by simple count-based retention (rulings record,
> premise amendment A2). This document is kept as **analysis and evidence**
> — the floor-mechanics reasoning, the I9-shaped pinning-test argument, and
> the error taxonomy it worked out are cited by `w2-spec.md` and
> `w3-spec.md` as mechanism detail the rulings do not contradict — not as a
> live design. Do not read anything below this notice as current behavior;
> read the rulings record and the two ADRs above instead.

Status: **Superseded**, 2026-08-21 — see disposition header above.
Date: 2026-08-20 (revised the same day after evaluation-gauntlet findings)
Audit basis: `miztertea/sergeant-rs` `backlog/w5-archival-design` @ `a730983c`
(integration branch carrying waves 1–2 of the backlog close-out sprint)
Scope: Journal segment retention, the replay contract's floor, §26 command
idempotency across that floor, blob-store liveness, DuckDB growth,
`sgt doctor`'s growth axis, and the explicit maintenance verb
Product behavior: Changed — the journal gains an archive tier and a
non-zero replay floor; below-floor reads gain a declared contract; §26
exact-once degrades below the floor from replayed-body to named-refusal;
nothing else is deleted, and no automatic action is added

**Revision note.** The first draft of this document was evaluated and
returned seven confirmed findings, two of them blockers: a naive replay
floor would have silently re-armed duplicate-Work creation through the §26
command ledger, and would have broken `sgt work transcript`, default
`/v1/events`, SSE, and `sgt doctor` outright on the first archived segment.
Both are resolved here (§3.1, §4.4), along with a false crash-safety claim
(§4.3), an undercounted full-replay consumer (§1.5), an unbounded guard-hold
in the verb itself (§4.8), a doctor-versus-archival race (§4.8), and one
arithmetic slip (§1.3). Where the fixes cost complexity, this document says
so at the point it is incurred rather than in a summary; where the earlier
draft was wrong, it says that too, because the *shape* of what it missed —
persisting one derived structure and assuming that exhausted the set — is
the failure mode I9 and its pinning test now exist to prevent.

────────

## 0. Relationship to existing decisions and defects

| Existing decision or defect | Disposition here |
|---|---|
| `docs/DEVELOPMENT.md:37`: "the journal is the only truth… rebuild-on-start is the only population path; there is no snapshot loading (backlog B1 explains why)" | **Preserved, with one narrow amendment proposed.** Archived segments stay inside the replay contract. The only derived state this proposal persists is the slim index row (§4.3) — seven scalar fields per Work, all re-derivable, binding-checked against the archive. B1's general snapshot machinery stays dormant. |
| ADR 0003 (durability promise) | **Not amended by the recommended composition.** ADR 0003 promises *resumability*, not never-delete (§2.6). A rung that discards history would amend it; none of the adopted rungs do. The tripwire is pre-committed in §2.6 so a later rung cannot cross it silently. |
| Retention ruling, 2026-08-11 (Rule C, amended at adjudication): binding trigger is rebuild-on-start > 30 s; ladder is compress-cold-segments-first, then snapshot/truncate | **Honored, with one correction offered for ruling.** Compression is adopted — as a rider on archival rather than as a standalone rung — but §3.5 shows compression addresses *disk*, while the ruled trigger is *time*. A cheaper rung 0 (§4.1) sits below both. Open question Q1. |
| Retention ruling, Rule B: `sgt doctor` disk-pressure check | **Extended.** The shipped check (`src/cli.rs:3018`, thresholds at `3042-3043`) measures *free space*, not *growth* — it cannot see a data dir growing to 500 GB on a disk with 600 GB free (§3.4). A growth axis is added beside it. |
| Retention ruling, Rule B: blob GC deferred, trigger = 20 GiB blob share or the first real deletion policy question | **Kept deferred, and deflated honestly.** §3.3 shows mark-sweep as constrained by this design reclaims only never-referenced blobs, because archival never deletes history. The real blob lever is a policy question the owner must answer (Q6), exactly as Rule B predicted. |
| Issue #4 / wave 2's bounded terminal-Work cache (`src/runtime/projection.rs:366,378`) and its miss path (`projection.rs:1141-1150`) | **Load-bearing new constraint, and a beneficiary.** The miss path is a *full* journal replay under the core guard. At 1M events that is a ~26–34 s stall on `sgt work show` (§1.5). The archive manifest's per-segment work-id set turns it into a two-segment read — ~35× less time under the guard (§4.6). |
| §26 command idempotency — `WorkRegistry::commands` / `command_works` (`projection.rs:246-280`), consulted by `replay_command` at the top of every mutating handler (`api.rs:1027-1080`) | **Safety-load-bearing, and it constrains the design.** A naive replay floor would drop the ledger for archived history and silently re-arm duplicate-Work creation on a retried `command_id`. The manifest carries the command *keys*; below-floor retries are refused by name, never re-executed (§3.1, Q8). |
| `sgt work transcript` (`api.rs:2173-2200`) — a full from-seq-0 replay under the core guard, already disclosed as unbounded in its own doc comment | **Third full-replay consumer, named.** Rung 3 shrinks its input to the segments the manifest names, and §7 states plainly that this is a mitigation, not the "journal reader the core does not own" that the comment says a real fix needs. |
| Issue #159 / estate-root contract §12.1, §12.3: durable artifacts are never auto-deleted; explicit deletion is a separate maintenance action | **Followed, with one named exception.** `sgt journal archive` previews by default and acts only on `--yes`. Startup reconciliation completes an unlink that a crash interrupted — an operator-authorized act finished later, the same shape as `recovery.rs`'s interrupted teardown (I4, Q9). |
| `sgt work reap --yes` (`src/cli.rs:422`, `POST /v1/work/{id}/reap`) | **Precedent adopted** for the verb's shape: explicit, confirmed, reported per item, never scheduled. |
| ADR 0008 (manifest authority over storage paths) | **Respected.** The archive lives inside the already-resolved data dir (`<data-dir>/journal/archive/`). No new manifest key, no new resolution rung. |
| ADR 0012 (estate and doctor are daemon API surface) | **Binding.** The journal holds an exclusive advisory lock for the daemon's lifetime (`src/runtime/journal.rs:68,256`), so the archive verb cannot be an offline file mover. It is a daemon API call (§4.8). |
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
50,000 / (1.71 − 0.44) = 39.4k ev/s.) Cerberus measured 54.6k ev/s
(`docs/perf/baseline-cerberus-2026-08-11.md`, S5: 50k events in 924.09 ms →
54,553 ev/s), which puts the same mark at 18.3 s. So across one known
platform spread, **1M events is 18–34 s of cold start — at or past the ruled
30 s trigger, not a safe distance from it.** The 2026-08-11 ruling estimated the trigger at
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

### 1.5 The constraints that did not exist when Rule C was deferred

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

**And it is not the only one.** `sgt work transcript` is a *third*
full-replay consumer under the same guard, and the codebase already
discloses it in its own doc comment (`src/api.rs:2173-2186`):

> `events_after(0)` below runs a full from-seq-0 journal replay while
> `core` — the exclusive `CoreGuard` — is still held… `resolve_run`'s
> `terminal_runs` cache accepts the identical shape only as a rare,
> capacity-bounded cache-miss fallback; **there is no equivalent bound
> here, because a work's conversation history is unbounded and not capped
> by any terminal-state cache.** Closing this for good needs a journal
> reader the core does not own.

So the honest count is **three** consumers of full replay, not two, and
transcript is the *worse* of the two guard-held ones: the #4 miss path is at
least rate-limited by a 1024-entry cache, while `work_transcript` takes a
full replay on **every** call, for any Work, hit or miss, and its cost is
uncapped by design.

An earlier draft of this document said "a second consumer" and sold rung 3
solely on `rederive_registry_for`. That was the exact failure this document
criticizes elsewhere — presenting a partial fix as if it exhausted the
problem — and it is corrected here. §4.6 extends the fix to transcript and
states plainly what it does and does not close.

Any archival design that makes any of these three paths worse is
disqualified. That rule now has teeth it did not have in the earlier draft:
§4.4 exists because a naive floor makes `work_transcript` fail outright.

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

**The earlier draft of this document proposed copying
`Journal::replay_after` as the model for a non-zero floor. That was wrong,
and the way it is wrong is the sharpest mechanical hazard in the whole
design.** `Replay::after` recomputes `expected` from a real segment's
`first_seq` *only* when some segment satisfies `first_seq <= after + 1`
(`journal.rs:548-565`). When every live segment begins **above** `after + 1`
— which is precisely what archival produces for every below-floor `after` —
the loop breaks on its first iteration, `keep` and `expected` are never
assigned, and `expected` keeps its initialized value of `1`. The next
`Replay::next()` then compares the first live event's real seq (580,679 in
§4.8's worked example) against `expected == 1` and fails with
`SeqDiscontinuity`.

`Replay::new` (`journal.rs:473-477`, and every other construction path)
hardcodes the same `expected = 1`.

So the root cause generalizes: **`Replay` has exactly one code path that can
ever expect a seq other than 1, and archival is precisely the condition that
makes that path not fire.** The moment any segment is archived, every reader
that does not go through the manifest gets a spurious hard error.

That is not a corner case. `events_after(0)` is what `sgt work transcript`
calls unconditionally (`api.rs:2196`), what `GET /v1/events` defaults to
(`EventsQuery::from` defaults to 0, `api.rs:3322-3325`), what a fresh SSE
connection without `Last-Event-ID` uses (`api.rs:3391-3399,3455`), and
`replay_data_dir` — doctor's lock-free path — is a bare `Replay::new`
(`journal.rs:473-477`). All four break on the first archival pass.

**Therefore I3 is stated as a contract, not as a mechanism:**

> Every `Replay` construction takes its expected floor **explicitly**. No
> construction path may infer `1`. A request for events below the floor is
> either served from the archive through the manifest, or answered with a
> named `ArchivedRange` outcome the surface renders honestly — **never**
> with `SeqDiscontinuity`, which keeps its single existing meaning: the
> history on disk is corrupt.

§4.4 specifies the behavior for every one of those consumers by name.

### 2.4 I4 — Nothing is deleted except by an explicit maintenance action

The estate contract's rule ("Deletion is a separate explicit maintenance
action with its own authorization and dry-run evidence",
`docs/proposals/estate-root-git.md:971-973`) and the shipped precedent
(`sgt work reap --yes`) bind here. No automatic reaper, no scheduled sweep,
no on-start compaction, no deletion as a side effect of any other verb.

**One named exception, and it must be named rather than left implicit.**
Startup reconciliation (§4.3) completes the unlink of a live-directory
segment whose archived counterpart is present and hash-verified — a deletion
the daemon performs without anyone typing anything at that moment. It is
permitted because it *completes an operator-authorized action interrupted by
a crash*, which is exactly what `src/runtime/recovery.rs` already does for an
interrupted surface teardown: recovery acts on evidence that something was
left unfinished. It is not permitted to reclaim anything nobody authorized —
which is the same line `recovery.rs:118-123` draws when it refuses to delete
stranded surface directories and defers them to a garbage collector. Q9 asks
the owner to ratify that distinction explicitly.

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

**Corollary the earlier draft missed:** doctor is a *second, unsynchronized*
reader of the same directory. `journal_check` calls
`Journal::replay_data_dir(data_dir)` directly (`cli.rs:3290`), bypassing both
the `CoreGuard` and the OS lock, and that function's doc comment
(`journal.rs:468-477`) tolerates exactly one race — a torn last line of the
live segment — and declares "every other `Malformed`… is corruption, not a
race, and must fail closed exactly as it does today." Archival introduces a
second race class (segments renamed and unlinked underneath a lock-free
scan). Adding a mutation pass without extending that tolerance would make
`sgt doctor` report corruption during a perfectly healthy archival run.
§4.8 specifies the concurrency story.

### 2.9 I9 — The floor state must be complete, and completeness must be tested

This invariant exists because the earlier draft violated it. It persisted
the slim index rows so `work list` would survive archival, and stopped
there — missing that `WorkRegistry::catch_up` also folds the **command
idempotency ledger** (`commands` and `command_works`,
`projection.rs:264-280`, `1087-1100`), which is safety-load-bearing and not
disposable (§3.1).

> Any registry state that a floor-started replay can no longer derive is
> either (a) carried in the archive manifest's **FloorState**, or (b)
> explicitly declared out of contract with its consequence named. There is
> no third option, and the boundary is pinned by a test, not by prose.

The pinning test is specified in §6: fold the full journal and fold
floor+FloorState, then assert the resulting `WorkRegistry` values are equal
on every field, with an *enumerated allowlist* of fields permitted to
differ. A new registry field added without deciding its archival disposition
fails that test rather than shipping a silent hole.

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
  immediately violates I2 and I9: the state that `WorkRegistry::catch_up`
  folds from below-floor events simply stops existing.

**This is the crux of Rule C, and the 2026-08-11 ruling identified it
exactly:** "Segments interleave works, so archiving 'old' segments while
rebuild-on-start replays the full journal is a contradiction — segment
retirement requires a load-bearing snapshot/checkpoint, which is exactly the
B1 machinery ruled dormant."

The ruling is right that (a-ii) requires persisting something derived. What
this proposal adds is an accounting of *exactly what*. The reducer
`catch_up` drives (`apply_registry_event`) writes four things that outlive
the live window:

**1. `works` / `runs` — full Work and run state.** Bounded already by wave
2's caches; a floor-started replay simply doesn't populate them for archived
Works, and the miss path (§4.6) re-derives on demand. **Nothing to
persist.**

**2. `work_index` — the slim index row.** Six scalar fields (`id`,
`intent`, `state`, `integrity`, `created_at`, `updated_at`) plus one added
`last_seq` (§4.3). All terminal-immutable for exactly the Works that
qualify. ~200 B/row → 12.5 MB at 62,500 Works. **Persist.**

**3. `commands` — the §26 command idempotency ledger.** *This is the one the
earlier draft missed, and it is not disposable.* `CommandOutcome` records
the status and exact response body of every accepted or rejected mutating
command, and its doc comment states the promise plainly: a repeated
`command_id` "replays this record verbatim instead of re-executing —
including across a daemon restart, because the registry is rebuilt from the
journal" (`projection.rs:246-250`). `replay_command` /`record_and_respond`
consult it at the top of every mutating handler (`api.rs:1027-1080`).

**4. `command_works` — the crash-window index.** Maps `command_id` → the
Work id a submit created, and its own doc comment names the stakes: without
it "a client retry of the same `command_id` would look brand new and create
a *second* Work record, breaking exact-once for the one case §26 exists to
serve: retry after an uncertain outcome" (`projection.rs:270-280`).

**So a naive floor silently re-arms duplicate-Work creation.** Once the
segment holding a submit's `command.accepted` is archived, a cold start
after that point has no record of the command, and a client retrying that
`command_id` gets a *second* Work. This is a safety regression, not a
convenience one, and it is the reason §3.1's disposition is narrower than
the earlier draft's.

**Three ways to answer it, and why the third wins:**

- **Exclude command events from eligibility.** Every mutating command
  journals one, so command events are spread through essentially every
  segment. This makes almost nothing archivable. **Dead on arrival.**
- **Persist the full `commands` map.** `CommandOutcome.result` is the entire
  original response body — for a submit, the whole Work JSON. At ~4 commands
  per Work and 1–2 kB each, 62,500 Works is 250k entries and 250–500 MB of
  manifest. That does not bound growth; it relocates it. **Rejected.**
- **Persist the command *keys*, not the outcomes.** The manifest carries the
  archived `command_id` set, plus the `command_id → work_id` mapping for
  submits (two ULIDs plus JSON framing, ~90-100 B/entry for submits and ~60 B otherwise; ~15-20 MB at 250k commands — the same order
  as the index rows). A retried below-floor `command_id` is then **refused
  with a named error, never re-executed**, and for a submit the refusal
  names the Work that command already created. **Adopted.**

The honest cost: below the floor, §26 degrades from "replay the identical
response" to "refuse, and name the Work if there is one." The **safety**
property — never re-execute, never create a duplicate Work — is preserved
exactly. The **convenience** property — a byte-identical replayed body — is
lost. That is a contract change to §26 and it is Q8, not an implementation
detail.

Together these are the manifest's **FloorState** (§4.3), and I9 requires a
test that pins its completeness rather than trusting this list to have been
exhaustive — because this list was demonstrably not exhaustive one draft ago.

**Disposition: ADOPTED**, as (a-ii) with the narrowed durable artifacts
above, gated on the owner permitting the narrowing (Q2) and the §26
degradation (Q8). If either is ruled against, this collapses to (a-i) and
Rule C can only ever address disk, never rebuild time.

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

Five rungs, cheapest-first (§4.1, §4.2, §4.3, §4.6, §4.7). Each is separately
shippable and each is useless-but-harmless if the one above it is skipped.

Three sections between them are **not** rungs and are not optional:

- **§4.4, the below-floor read contract**, is a hard prerequisite for §4.3.
  Archival cannot ship before it, because the first archived segment breaks
  four working read paths without it.
- **§4.5** derives the one default the composition rests on.
- **§4.8** specifies the verb and its concurrency obligations.

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
    manifest.ndjson
    .staging/
    00000001.ndjson.zst
    00000002.ndjson.zst
    ...
```

Inside the already-resolved data dir; no new manifest key, no new resolution
rung (ADR 0008).

**`manifest.ndjson` is append-only**, folded at read time — the same shape as
the journal itself. This is not decoration: §4.8 commits one segment at a
time so the verb's guard-hold stays bounded, and a rewrite-the-whole-file
manifest would mean re-fsyncing a 10–20 MB document 38 times in the worked
example. An append is one line and one fsync.

Three record types:

```json
{"t":"segment","index":1,"first_seq":1,"last_seq":15281,"events":15281,
 "raw_bytes":8388401,"stored_bytes":998112,"codec":"zstd","blake3":"b3:…",
 "work_ids":["01H…","01H…"]}
{"t":"work","id":"01H…","intent":"…","state":"completed","integrity":"clean",
 "created_at":"…","updated_at":"…","last_seq":15044}
{"t":"command","command_id":"01H…","work_id":"01H…"}
{"t":"floor","archived_through_seq":15281,"segments":1,"blake3_chain":"b3:…"}
```

A `floor` line commits everything appended since the previous one. **The
floor is the last `floor` line's `archived_through_seq`, and nothing else in
the file is authoritative until a `floor` line follows it** — which is what
makes a crash mid-append harmless: a partial tail is discarded exactly the
way a torn journal tail is.

The `work` and `command` lines together are the **FloorState** (I9):

- `work` — the slim index row, plus `last_seq`, which is not part of
  `WorkIndexRow` today and is added because it is what lets a read skip the
  live window entirely (§4.6).
- `command` — the §26 ledger *keys* (§3.1): one line per archived
  `command_id`, carrying `work_id` when the command was a submit. This is
  what keeps a retried below-floor `command_id` from creating a duplicate
  Work. The `CommandOutcome` *bodies* are deliberately not carried; see §3.1
  for the cost of that and Q8 for the ruling it needs.

Both are produced by folding the **same** `apply_registry_event` arms the
live reducer uses, not by a hand-rolled subset — and I9's pinning test
(§6) is what keeps them from drifting apart.

The whole manifest is re-derivable by replaying the archived segments, which
is what makes it a checkable cache rather than a second truth (I1).

**Manifest growth, stated rather than waved at.** One `segment` line per
archived segment, one `work` line per archived Work, one `command` line per
archived command. At 62,500 Works / 250k commands / 65 segments that is
~22 MB, folded once at daemon start. It grows with history, like everything
else here — the difference is that it grows at ~1/40th the rate of the
journal it replaces. If an estate ever passes ~1M archived Works this
becomes its own Rule C problem in miniature; that is honest, and it is far
past any horizon in §1.3.

**Archival eligibility.** A segment is archivable if and only if:

1. it is not the currently-open segment; **and**
2. it is older than the live window (§4.5); **and**
3. every event in it carries a `work_id` whose Work is **terminal and
   settled** — the same `is_absorbing` + `run_is_settled` predicate wave 2
   already uses for cache eviction (`projection.rs:644,701`) — and whose
   `work` FloorState line is appended; **and**
4. every event in it *without* a `work_id` is of a kind on an explicit
   archivable allowlist (Q5); **and**
5. every `command.accepted` / `command.rejected` event in it has its
   `command` FloorState line appended (§3.1). Command events do **not** block
   eligibility — that variant was considered and is dead on arrival, since
   essentially every segment holds one — but a segment cannot be archived
   until its commands are in the FloorState.

Condition 3 is the ruling's "segments interleave works" objection answered
directly rather than argued around: a segment holding even one event of a
live Work is simply not eligible. **Named consequence: one long-lived Work
can stall archival indefinitely.** That is accepted, not worked around — the
`journal_growth` check reports the blocking Work by id so the operator sees
*why* the live journal is not shrinking, and the honest remedy is to finish
or cancel that Work.

**Replay contract change.** Every `Replay` construction takes its floor
explicitly (I3); none may infer 1. Per-consumer behavior is specified in
§4.4. Two mechanical notes belong here:

- **Full replay** (`Journal::replay_from_archive`, new) reads archived
  segments in index order, verifying each against its recorded `blake3`
  before decoding, then continues into the live segments — one continuous
  seq chain from 1, exactly as today.
- Torn-tail classification (`journal.rs:87-101,166-175`) is unaffected:
  archived segments are immutable and closed, so a torn tail is still only
  ever possible on the last line of the last live segment — and on the last
  line of `manifest.ndjson`, which is discarded the same way.

**Startup reconciliation.** The earlier draft claimed "manifest before
unlink, so a crash at any point leaves either the original or a
manifest-listed archive, never a hole." That claim was false in one window.
A crash between the `floor` line's fsync and the original's unlink leaves
the original segment **still in the live directory and already below the
floor**. `list_segments` would return it as the lowest-index live segment,
its `first_seq` far below `archived_through_seq + 1`, and floor validation
would then refuse to start the daemon — the exact state a §6 acceptance
criterion says cannot exist.

So the step is specified rather than assumed. `Journal::open`, before
validating the floor:

1. folds `manifest.ndjson` to the last committed `floor` line, discarding
   any uncommitted tail;
2. for each live-directory segment whose entire `[first_seq, last_seq]`
   range is `<= archived_through_seq`: verifies the archived counterpart is
   present and its `blake3` matches, then **completes the pending unlink**;
3. reports any live segment below the floor whose counterpart does *not*
   verify as a hard, named failure — that is corruption, not an interrupted
   move;
4. only then validates that the first remaining live segment's `first_seq`
   equals `archived_through_seq + 1`.

This is a deletion the daemon performs unprompted, which is why I4 names it
as an explicit exception: it completes an operator-authorized action
interrupted by a crash, the same shape as `recovery.rs` finishing an
interrupted teardown. Q9 asks the owner to ratify that reading.

The reverse window — after the staged file is renamed into `archive/` but
before the `floor` line commits — is benign by construction: the original is
still live, the floor has not moved, and the orphan in `archive/` is `sgt
journal verify`'s "present but unlisted" class. Reconciliation never deletes
from `archive/`; it only ever completes a pending unlink of a *live* segment
whose archived twin hash-verifies.

**Cost of this, stated:** the crash-safety story is no longer "one atomic
rename." It is a two-phase commit with a reconciliation pass, which is more
machinery than the earlier draft implied and is one more thing that has to
be crash-tested (§6).

**Compression** rides the move: a segment is compressed as it is archived
(`.ndjson.zst`), never in place, never on a live segment. This is the
2026-08-11 ladder's rung 1, folded in at zero extra machinery. Expected
5–10× on NDJSON with repetitive keys — **unmeasured**, §5.4.

**Nothing is deleted** except the live copy of a segment whose compressed
twin is on disk and hash-verified. Archival is a move plus a compress. The
bytes stay.

### 4.4 The below-floor read contract

I3 says no `Replay` may infer a floor of 1 and no below-floor read may
surface as `SeqDiscontinuity`. This section discharges that for **every**
from-seq-0 or below-floor consumer in the tree, by name. The earlier draft
specified none of them, and would have shipped a spurious hard error on
`sgt work transcript`, `GET /v1/events`, SSE, and `sgt doctor` the moment
the first segment was archived.

| consumer | call site | today | after archival |
|---|---|---|---|
| cold-start registry fold | `daemon.rs:460` | `replay()` from 1 | floor replay + FloorState fold (§4.3) |
| DuckDB rebuild | `daemon.rs:467` | `replay()` from 1 | floor replay; analytics below the floor is **declared out of contract** (§7) and doctor says so |
| #4 miss path | `projection.rs:1141` | full replay under guard | manifest-directed archived read (§4.6) |
| `sgt work transcript` | `api.rs:2196`, `events_after(0)` | full replay under guard | manifest-directed archived read for that Work (§4.6) |
| `GET /v1/events` (default `from=0`) | `api.rs:3322-3325` | full replay | live window + `truncated_below` marker |
| SSE without `Last-Event-ID` | `api.rs:3391-3399,3455` | `events_after(0)` | live window + one leading `archived-range` event, then live |
| SSE with a below-floor `Last-Event-ID` | same | `replay_after(n)` | same marker; the client learns its resume point is gone |
| `sgt doctor` journal check | `cli.rs:3290` → `replay_data_dir` | `Replay::new`, expects 1 | manifest-aware floor, with the concurrency rule of §4.8 |

Three rules generate that table.

**R1 — the floor is always explicit.** `Replay` takes `expect_from` at
construction. `Journal::replay()` passes the manifest floor;
`replay_from_archive()` passes 1; `replay_after(n)` passes
`max(n + 1, floor)` and keeps its existing segment-skipping on top. The
`expected = 1` default disappears from the type. This one change is what
makes the rest of the table possible, and it is the smallest possible fix
for the root cause named in I3.

**R2 — a below-floor request is answered, not refused.** Two shapes,
chosen by what the caller is actually asking:

- **Per-Work reads** — `work show`, `work transcript` — ask *what happened*,
  and the manifest can answer: the `work` line's `last_seq` says whether the
  live window is even needed, and the `segment` lines' `work_ids` say which
  archived segments to read. These get real history, not a marker (§4.6).
- **Stream reads** — `/v1/events`, SSE — ask *what is happening*. They get
  the live window plus an explicit `truncated_below: <floor>` field (SSE: one
  leading `archived-range` event before the history replay). The surface
  renders "events before seq N are archived" rather than pretending the
  stream starts at the beginning.

Q10 asks the owner to ratify that split, because it is a product decision as
much as a mechanical one.

**R3 — one named error, distinct from corruption.**
`JournalError::ArchivedRange { requested_after, archived_through }` is
returned when a caller genuinely needs below-floor events and the archive
cannot serve them — a missing or unverifiable archived segment. It surfaces
as HTTP 409 `archived_range` with a remedy naming `sgt journal verify`.
`SeqDiscontinuity` keeps exactly one meaning, unchanged: **the history on
disk is corrupt.** Conflating the two is what the earlier draft would have
done by accident, and it is the difference between "some old history moved"
and "your journal is broken."

**Cost, stated.** This is real API surface: a new error variant, a new
response field on two endpoints, a new SSE event type, and a client-visible
behavior change for `/v1/events?from=0`. It is the price of archival being
honest instead of silent, and it is why §6 tests every row of the table
above rather than only the cold-start path.

### 4.5 The live-window default, argued from platform posture

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

### 4.6 Rung 3 — a manifest-directed per-Work read, for both guard-held paths

This is the argument that makes rung 2 worth its complexity even before the
rebuild-time benefit lands. It applies to **both** of §1.5's guard-held
consumers, not just the one the earlier draft named.

The shared mechanism — one lookup, used by both:

1. `work_index` (or the `work` FloorState line) confirms the id exists
   (already the case, `api.rs:1522`);
2. the manifest's `segment` lines' `work_ids` name the archived segments
   containing that Work — typically one or two, because a Work's events are
   contiguous in time;
3. the `work` line's `last_seq` decides whether the live window is needed at
   all: if `last_seq <= archived_through_seq`, no live segment can contain an
   event for that Work, so none is read;
4. the read covers only the named segments, filtered as today.

**For the #4 miss path** (`rederive_registry_for`, which today walks every
event and discards ~99.99% of them): at 1M events with a 16-segment live
window, for a fully-archived Work, **two archived segments (~30k events)
instead of 1M** — roughly 0.8 s instead of 26–34 s, a ~35× reduction in
guard-hold. A Work inside the live window is unchanged. A Work spanning the
floor fails step 3 and reads the live window too: ~7 s at that scale, still
4× better, and the rarest of the three cases.

**For `sgt work transcript`** (`api.rs:2196`), the same lookup replaces
`events_after(0)` with a per-Work archived read. The arithmetic is the same,
but **what it closes is not**, and the difference must be stated:

- The #4 miss path becomes *bounded* — the manifest names a small, fixed set
  of segments and the cache absorbs repeats.
- `work_transcript` becomes *proportional to one Work's own history* instead
  of proportional to all history. That is a large improvement and it is not
  a bound: the code comment at `api.rs:2173-2186` is right that "a work's
  conversation history is unbounded," and a single deep Work (S3 measured
  1,010 events on one) still reads every segment it touches, under the guard.
- The comment says closing this "for good needs a journal reader the core
  does not own." **This design does not deliver that reader.** It shrinks the
  input; it does not release the guard. Rung 3 is a mitigation for
  `work_transcript`, not a fix, and it should not be sold as one.

Recording `work_ids` and `last_seq` costs nothing extra: the archival pass
already reads every event in every candidate segment to evaluate eligibility
condition 3.

### 4.7 Rung 4 — blob liveness, kept deferred

No blob deletion ships. Rule B's trigger stands unchanged. What ships is the
groundwork that makes the eventual sweep cheap and correct:

- the per-segment `work_ids` set (already required by §4.6);
- an acceptance test that **no code path writes a blob without a journal
  event naming it** — Rule B's own acceptance clause, restated because the
  mark pass's correctness rests entirely on it;
- an acceptance test that a generic `b3:<64hex>` payload walk marks a blob
  written by *every* producer path (`claude.rs` `raw`, `docker.rs` `stdout`
  and `stderr`), so the untyped-payload hazard of §3.3 is pinned by a test
  rather than by a comment.

`sgt doctor` reports blob share against Rule B's 20 GiB backstop (§4.2). If
that fires, a blob-GC design contract fires with it — as already ruled.

### 4.8 The maintenance verb

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

**The pass must not hold the core guard for its own duration.** The earlier
draft took the guard in step 1 and held it through compress, fsync, rename,
and unlink for every segment — 38 segments and 304 MiB in the worked example
above. A document that spends four sections establishing that a guard-held
stall is a daemon-wide, disqualifying failure mode cannot then recommend, in
a doctor remedy line, a verb whose own guard-hold is unexamined. §1.5's rule
("any design that makes these paths worse is disqualified") applies to this
design too.

So the pass is windowed. **Per segment**, not per pass:

*Outside the guard* — the expensive part, and it needs no exclusivity
because an archival candidate is by definition never the open segment, so
its file is closed and immutable:

1. read the segment, evaluate eligibility, and fold its `work`/`command`
   FloorState rows;
2. compress into `archive/.staging/<index>.ndjson.zst`, `fsync` it.

*Inside the guard* — bounded, a handful of metadata operations independent
of segment size:

3. re-verify the segment is still a candidate (nothing archived it
   meanwhile, no Work un-settled);
4. rename staging → `archive/`, `fsync` the directory;
5. append the `work`/`command`/`segment` lines and a `floor` line to
   `manifest.ndjson`, `fsync`;
6. unlink the original.

*Then release the guard* before starting the next segment.

7. Once the pass finishes, journal `journal.archived` with the seq range,
   segment count, and byte counts, so the act is itself in the history it
   archives.

Guard-hold therefore scales with **segment count × a few fsyncs**, not with
bytes, and other requests interleave between segments.

**What this costs, stated.** Three things:

- **The pass is no longer atomic across segments.** A crash or a
  cancellation mid-pass leaves a partially archived journal. That is safe by
  construction — the floor advances one committed segment at a time and
  every intermediate state is valid — but "archival is all-or-nothing" is no
  longer true, and the preview should say how far a partial run got.
- **One manifest commit per segment**, which is exactly why the manifest is
  append-only NDJSON (§4.3) rather than a rewritten JSON document. 38
  appends and 38 fsyncs, not 38 rewrites of a 22 MB file.
- **The guard-hold is bounded but not measured.** Six fsync-class operations
  per segment on an unknown filesystem is an estimate, not a number. §5.6
  measures it, and until it does, this is a designed-for bound rather than a
  demonstrated one.

**`sgt journal verify`** re-reads every archived segment, checks its
`blake3`, and checks that the seq chain across archive-plus-live is
contiguous from 1. Its anomaly classes are:

1. an archived segment listed in the manifest but absent;
2. an archived segment whose bytes no longer match its `blake3`;
3. a manifest `segment` line whose file is absent (same as 1, named
   separately because the remedy differs: restore vs. re-archive);
4. a file present in `archive/` that no `segment` line names — the benign
   orphan of an interrupted pre-commit crash;
5. **a segment present in the live directory whose range is already below
   the floor** — the pending-unlink state of §4.3. Reported as
   *reconcilable*, not corrupt, with the remedy "restart the daemon; startup
   reconciliation completes it." Verify never performs the unlink itself,
   because verify is a read verb.

Class 5 is the one the earlier draft's four-class list could not express,
which is how the crash-window contradiction went unnoticed.

**Doctor versus a live archival pass.** `sgt doctor`'s `journal_check` calls
`Journal::replay_data_dir` directly (`cli.rs:3290`) — no `CoreGuard`, no OS
lock — and that function tolerates exactly one race today: a torn last line
of the live segment (`journal.rs:468-477`). Archival adds a second race
class, and without a rule for it a healthy archival run would make doctor
report corruption. The rule:

- **Order the reads so the common race is benign.** `replay_data_dir` lists
  segments **first**, then folds the manifest, then discards any listed
  segment lying entirely below the floor it just read. A floor that advanced
  between the two reads then produces a skip, not a gap.
- **Retry once on a vanished segment.** A listed segment that is gone when
  opened is treated exactly as the existing torn-tail race is: retry the
  whole `replay_data_dir` once. A second failure is corruption and fails
  closed, unchanged. This extends the function's tolerance doc by exactly one
  named case — the minimum honest change.
- **A quiescence marker is rejected.** Having the lock-free reader wait for
  the daemon to be idle would reintroduce the synchronization
  `replay_data_dir` exists to avoid, and would make `sgt doctor` block on a
  long archival run.

Cost: doctor's journal check can perform two full lock-free scans in the
rare racing run, roughly doubling that check's cost while an archival pass
is in flight. `journal_growth` reports when it retried, so the cost is
visible rather than mysterious.

────────

## 5. Measurement plan — what to measure before implementing

The issue names one measurement ("measure rebuild at 1M events"). It needs
six. Every one runs on the committed harness (`scripts/perf/`), which
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

The live-window default (§4.5) rests on an *assumed* 2× penalty for the
slowest platform target, because neither macOS nor WSL2 has ever been
measured for replay. Run `s5-journal.sh` at reduced marks
(`PERF_S5_MARKS="50000 250000"`) on a macOS host and inside a WSL2 distro
with the data dir on the Linux filesystem (not `/mnt/c` — ADR 0003 D6 makes
that configuration a hard refusal, not a slow one).

**What it decides.** Whether 16 segments / 128 MiB is right, generous, or
too small. Per ruling 7 this is the measurement that lets the default be
argued for the platform targets rather than assumed from one host.

### 5.6 The archive verb's own guard-hold and compress-side throughput

§4.8 bounds the guard-hold to "segment count × a few fsyncs" by design. That
is an argument, not a measurement, and this document disqualifies other
designs on exactly this axis — so it must measure its own.

On the 1M-event corpus from §5.1, run a realistic first archival pass (the
worked scale: ~38 eligible segments, ~304 MiB) and record:

- compress-side wall time per segment, outside the guard (read + zstd +
  fsync) — this is what makes the *pass* slow, and it is fine that it is;
- **guard-held wall time per segment** (re-verify + rename + dir fsync +
  manifest append + fsync + unlink) — p50/p95/max;
- total guard-held time summed across the pass, and the longest single
  uninterrupted hold;
- concurrent request latency (`sgt status`, a submit) sampled throughout the
  pass, which is the number an operator actually feels.

**What it decides.** Whether the windowed pass is genuinely invisible to
concurrent traffic, or whether per-segment metadata commits on a slow
filesystem add up to a stall of their own. If p95 guard-hold is not small,
the manifest commit batches across K segments and reconciliation tolerates K
pending unlinks — a change §4.3's reconciliation step already accommodates.

### 5.7 What is NOT measured first

The #4 miss-path cost (§1.5) needs no new measurement — it is a full replay
by inspection, so §5.1's rebuild figure *is* its figure, minus the DuckDB
fold. The same is true of `work_transcript`. Both are called out because
doctor should report them (§4.2), not because they are uncertain.

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
- A manifest whose committed floor does not equal
  `first live segment.first_seq − 1` **after startup reconciliation** is a
  named hard failure at daemon start.
- An uncommitted tail on `manifest.ndjson` (lines after the last `floor`) is
  discarded exactly as a torn journal tail is, and the daemon starts.
- Torn-tail classification is unchanged: `possible_torn_tail` is still true
  only for the last line of the last **live** segment
  (`journal.rs:87-101,166-175`).
- `list_segments` (`journal.rs:682`) never returns an archived segment, and
  rotation never reuses an archived index — `MAX_SEGMENT_INDEX`
  (`journal.rs:74`) continues to bound the monotonic counter, which
  archival does not reset.

### FloorState completeness (I9) — the pinning test

- **The generative test:** seed a journal, archive a prefix, then assert
  that folding `full replay` and folding `floor replay + FloorState` produce
  `WorkRegistry` values equal on **every** field, against an *enumerated
  allowlist* of fields permitted to differ. Adding a field to `WorkRegistry`
  without deciding its archival disposition must fail this test. This is the
  criterion that would have caught the command-ledger hole in the earlier
  draft, and it is the most important test in this list.
- The allowlist starts as exactly `{ commands (values only) }` — keys are
  carried, `CommandOutcome` bodies are not (§3.1, Q8) — and `works`/`runs`
  for archived Works, which are re-derived on demand (§4.6).
- `commands` **keys** and `command_works` after floor replay + FloorState
  equal those after full replay, exactly.
- FloorState rows are produced by the same `apply_registry_event` arms the
  live reducer uses — asserted by construction, not by a parallel
  implementation.

### §26 exact-once across the floor

- A submit whose `command.accepted` is below the floor, retried with the
  same `command_id` after a cold start, creates **no second Work** — it is
  refused with the named error and the response names the existing Work id.
- The same holds for the crash-window shape: `work.submitted` archived,
  `command.accepted` never written, retry after restart still resolves to
  the existing Work via the archived `command_works` mapping.
- A non-submit mutating command (`cancel`, `reap`) retried below the floor
  is refused, never re-executed.
- A command *above* the floor still replays its byte-identical body,
  unchanged from today.

### Below-floor reads (§4.4)

- Every row of §4.4's consumer table has a test at a seeded archived scale.
- **No** below-floor read returns `SeqDiscontinuity`. A test asserts the
  specific error/marker per consumer.
- `sgt work transcript` on a fully-archived Work returns the same events it
  returned before archival.
- `GET /v1/events` with no `from` returns the live window plus
  `truncated_below` equal to the floor.
- A fresh SSE connection with no `Last-Event-ID`, and one with a below-floor
  `Last-Event-ID`, both receive the `archived-range` marker and then live
  events — neither errors, neither silently starts mid-history.
- `Replay` has no construction path that defaults `expected` to 1 —
  asserted by the type's signature, not by a runtime check.

### Archival correctness

- A segment containing any event of a non-terminal or unsettled Work is
  never archived, and the blocking Work is named in the preview.
- A segment containing a non-work-scoped event whose kind is not on the
  allowlist is never archived.
- Crash injection at every point in the archival pass — after compress,
  after staging fsync, after rename into `archive/`, after the `floor`
  append, before unlink, and mid-pass between segments — leaves a journal
  that passes `sgt journal verify` and a daemon that starts. The
  after-`floor`-before-unlink case specifically must exercise startup
  reconciliation.
- Startup reconciliation completes a pending unlink only when the archived
  twin hash-verifies; a below-floor live segment whose twin does *not*
  verify is a named hard failure, not a silent deletion.
- `sgt journal verify` detects each of its five anomaly classes (§4.8),
  including class 5 (pending unlink), and reports class 5 as reconcilable
  rather than corrupt.
- The pass journals `journal.archived`; a replay of the post-archival
  journal contains that event.
- Archival deletes no bytes beyond the live copy of a hash-verified archived
  segment. A byte-count assertion before and after: raw archived bytes are
  recoverable by decompression, and no blob is touched.

### Concurrency

- A `sgt doctor` loop running continuously against a live archival pass over
  ≥8 segments reports healthy on every iteration and **never** reports
  corruption or a seq gap.
- `replay_data_dir` lists segments before folding the manifest, and discards
  listed segments entirely below the floor.
- A segment that vanishes between listing and open triggers exactly one
  retry; a second failure fails closed as corruption, unchanged.
- The archive pass's guard-hold per segment is asserted against a budget in
  a test, so a regression to whole-pass guard-holding is caught.
- Concurrent submits and reads complete normally throughout an archival
  pass.

### Guard-held per-Work reads

- `rederive_work` on a Work whose events are entirely archived reads only
  the segments the manifest names for it, and returns a `Work` identical to
  the pre-archival re-derivation.
- `work_transcript` on such a Work likewise reads only the named segments,
  and returns events identical to the pre-archival call.
- Both paths' replay budgets are asserted in tests at a seeded scale, so a
  regression to full-replay is caught for either.

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
the gate the 2026-08-11 ruling set on Rule C and it blocks issue 5.

**4. `[journal] Give Replay an explicit floor and specify every below-floor read`**
`Replay::new` hardcodes `expected = 1` and `Replay::after` only overrides it
when a segment's `first_seq <= after + 1` (`journal.rs:548-565`), so the
first archived segment would make `sgt work transcript` (`api.rs:2196`),
default `GET /v1/events` (`api.rs:3322`), fresh SSE (`api.rs:3391,3455`), and
doctor's `replay_data_dir` all fail with a spurious `SeqDiscontinuity`. Take
the floor explicitly at every construction, add a distinct `ArchivedRange`
outcome, and specify each consumer per §4.4's table. **This is a hard
prerequisite for issue 5 — archival cannot ship before it.**

**5. `[journal] Implement segment archival: archive/ + manifest + FloorState + sgt journal archive`**
Implement §4.3–§4.8 of `docs/proposals/journal-archival-rule-c.md`: move
cold, fully-settled segments to `<data-dir>/journal/archive/` compressed,
record ranges/hashes/work-ids plus the FloorState (slim index rows and §26
command keys) in an append-only manifest, add startup reconciliation for the
pending-unlink crash window, and add the preview-by-default `sgt journal
archive` and `sgt journal verify` daemon verbs with the windowed guard-hold
of §4.8. Blocked on issue 3's measurements, issue 4's floor contract, and the
owner's rulings on Q2 (persisted FloorState) and Q8 (§26 degradation below
the floor).

────────

## 7. Explicit non-goals

This proposal does not:

- delete any journal event or any blob, automatically or otherwise (the one
  deletion in the design is the live copy of a hash-verified archived
  segment, §4.3);
- add a retention configuration surface to `sergeant.toml`;
- add any age-based or clock-based rule (the #4 ruling's "count-bound, no
  clocks" posture carries here);
- rewrite, renumber, or compact history (§3.2);
- introduce projection snapshotting in the general B1 sense — only the
  narrowed, re-derivable FloorState of §4.3, and only if Q2 permits;
- give DuckDB a retention policy of its own (it is deleted and rebuilt on
  every start, `analytics.rs:708`; it inherits the journal's horizon).
  **Declared consequence, per I5:** once a floor exists, DuckDB is rebuilt
  from the live window only, so analytics over archived Works returns no
  rows. That is data out of the analytical contract, and it must be said
  where a user meets it — `sgt analytics` and `journal_growth` both report
  the analytical floor, so an empty result reads as "archived," never as
  "never happened." Recovering it means a full `replay_from_archive`
  rebuild, which is available and slow, not impossible;
- close `work_transcript`'s unbounded guard-held read. Rung 3 shrinks its
  input by naming the segments; it does not hand the read to a journal
  reader outside the core, which is what `api.rs:2173-2186` says a real fix
  requires (§4.6);
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

**Q2. May the FloorState be persisted, or does B1 stay fully closed?**
This is the load-bearing question. Archival that skips replay of archived
segments *requires* a durable form of the registry state those events built:
the slim index rows **and** the §26 command-ledger keys (§3.1). If B1 stays
fully closed, archival can only ever save disk, never rebuild time, and Rule
C cannot address its own binding trigger.
*Recommendation: permit, narrowly.* Seven scalar fields per Work plus two
ULIDs per command, terminal-immutable for exactly the entries that qualify,
~22 MB at 62,500 Works and 250k commands, re-derivable at will by replaying
the archive, and binding-checked by the committed floor plus per-segment
BLAKE3. That is a cache with a mechanical proof of freshness, not a second
truth — which is the specific property B1's identity binding was dormant for
want of. The scope of what must be persisted grew between drafts, which is
why I9's pinning test matters more than this recommendation does: the answer
to "what else did we miss" has to be a test, not a promise.

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

**Q8. May §26's exact-once contract degrade below the archive floor?**
Above the floor, a retried `command_id` replays a byte-identical response.
Below it, this design carries only the command *keys* — so a retry is
**refused with a named error**, and for a submit the refusal names the Work
that command already created (§3.1). The alternative, persisting every
`CommandOutcome` body, would put 250–500 MB of response JSON in the
manifest and relocate the growth this proposal exists to bound.
*Recommendation: permit the degradation.* The **safety** property §26 exists
for — never re-execute, never create a duplicate Work — is preserved
exactly; only the byte-identical-replay convenience is lost, and only for
commands older than the entire live window (weeks of history), which is far
outside any plausible client retry horizon. But this is a change to a stated
contract, so it is the owner's to make, not the implementation's.

**Q9. Is completing an interrupted unlink at daemon start "automatic
deletion" under I4?**
Startup reconciliation deletes a live segment whose archived twin
hash-verifies, without anyone typing anything at that moment (§4.3).
*Recommendation: rule it permitted, and write the distinction into I4.* It
completes an action the operator authorized with `--yes` and a crash
interrupted — the same thing `recovery.rs` does for an interrupted surface
teardown. It is *not* permission to reclaim anything nobody authorized,
which is the line `recovery.rs:118-123` already draws when it refuses to
delete stranded surface directories. Left unstated, this reads as a
violation of the estate contract's deletion rule; stated, it is a bounded
exception with a precedent.

**Q10. What does a below-floor stream read return?**
`GET /v1/events?from=0` and a fresh SSE connection ask for history that no
longer starts where they assume. Three options: serve it from the archive,
return the live window with an explicit truncation marker, or refuse.
*Recommendation: split by what the caller is asking* (§4.4). Per-Work reads
(`work show`, `work transcript`) ask *what happened* and get real archived
history through the manifest. Stream reads ask *what is happening* and get
the live window plus a `truncated_below` marker, because serving a stream
subscriber the entire archive is the cost profile this whole proposal is
trying to eliminate. Either way the answer is never `SeqDiscontinuity` —
that error keeps its single meaning, which is that the journal is corrupt.

────────

## 9. Final statement

After this proposal lands, the growth story is:

```text
cold start replays one window, not the whole history
cold segments move to archive/ compressed, and stay replayable
the replay floor is explicit, hash-checked, and verifiable
no reader ever infers a floor of 1, and no below-floor read
  reports corruption — it is served, or it is named
a retried old command is refused by name, never re-executed
an evicted Work re-derives from two segments, not from all of them
sgt doctor reports growth and rebuild time, not just free space
sgt journal archive previews by default, holds the guard per rename
nothing is removed but the live copy of a hash-verified archive
```

And the invariant survives intact:

> **The journal is still the only truth. Archival changes where the truth
> is stored and how much of it a cold start must read — never whether it
> can still be read.**
