# 00-orient — #334: journal writes silently fail; the invariant is prose

## 1. Pinned fixed point (`@@pin-fixed-point`)

| | |
|---|---|
| Revision | `f2b7a3720fe99c3a2d112cf8296339c98fdcec1b` |
| Confirmed | `git rev-parse HEAD` → `f2b7a3720fe99c3a2d112cf8296339c98fdcec1b` |
| Subject | `Merge pull request #354 from miztertea/hats6/c1d-attribution` |
| Lane | `/var/tmp/hats6/journal`, clean at pin (`git status --porcelain` empty) |
| Crate version | `sergeant-rs v0.3.0` (from the `cargo nextest` build line) |

No diff bounds this run — this is an implement-change run whose fixed point is
the base the whole run is judged against. Nothing downstream re-pins (J5,
`@@pin-fixed-point`: *"Nothing downstream may re-pin."*).

**A correction to the brief, at the pin.** The brief (line 62) states commit
`335d5892` is *"on `hats6/scan-front-door`"* and the dispatch prompt states it is
*"already merged to the integration branch."* Against this pin that is false:

```
$ git merge-base --is-ancestor 335d5892 f2b7a372 && echo yes || echo no
no
$ git branch -a --contains 335d5892
+ hats6/scan-front-door
```

`335d5892` is **not** an ancestor of the pinned base. Its scan front door is not
in the code this run changes. Item 2 is answered below anyway, against the
branch's own tree, because it will merge — but no change in this run may assume
it is present. (J5: the pinned revision is the fixed comparison point; a stage
prompt cannot move it.)

## 2. Spec / acceptance source (`@@identify-spec-source`)

Priority order walked; two levels hit, both recorded by path:

1. **Explicit reference in the intent** — `gh issue view 334`, read in full.
   Three numbered acceptance items: (1) reproduce deterministically, (2) decide
   the contract on failed journal writes, (3) a test that a shutdown always
   journals its own stop event.
2. **Path supplied by the intent** — the wave brief,
   `knowledge/evidence/resources/host-atlas-s6-series/brief-334-journal-integrity.md`
   (five decisive-work items; items 1/3/4/5 in scope, item 2 escalated).
3. **Governing contract (J5, wins over both)** —
   - `knowledge/evidence/resources/host-atlas-series/A1-ATLAS-WORLD-INTELLIGENCE.md:627`
     (A1-01): *"Atlas is derived evidence; journal/Git/original bytes remain
     authority."*
   - same file, §12 (line 500-502): *"The daemon remains sole Atlas writer."*
   - `.../H1-HOST-RUNTIME.md:88-92` (§6, Central journal): *"The journal remains
     the authoritative evidence stream for what Sergeant can prove about its own
     execution."*

No conflict found between the brief and the contract on substance; the one
conflict is factual (the `335d5892` merge claim, §1) and the pin wins.

## 3. Reproduced — real captured output

Deterministic, offline, no clock. Driven through the same
`record_scan` → `Journal::append` path the defective handler uses
(`src/runtime/atlas/record.rs:292`, `let event = journal.append(draft)?;`), then
a `Core::commit`. Scratch suite kept at
`/var/tmp/sgt-test-tmp/orient_repro_334.rs.keep` (deliberately not committed —
the durable suite is `10-implement`'s to shape and to wire for #231).

```
$ CARGO_BUILD_JOBS=6 TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run \
    --test _orient_repro_334 --no-capture
before: registry.last_seq=1 journal.next_seq=2
after direct append (no absorb_journaled): registry.last_seq=1 journal.next_seq=3
NEXT COMMIT FAILED: projection seq mismatch: expected 2, got 3
        PASS [   0.137s] a_direct_journal_writer_that_skips_absorb_breaks_the_next_commit
control: absorb_journaled keeps the next commit green
        PASS [   0.151s] the_same_writer_with_absorb_journaled_does_not_break_the_next_commit
```

The error string is `projection seq mismatch: expected 2, got 3` — the exact
shape of the issue's second observation (`expected 156, got 157`). The control
case is the non-vacuity proof: the identical fixture with `absorb_journaled()`
inserted commits cleanly, so the guard is measuring the absorb and not the
fixture.

### 3a. The cascade — this is where the issue's *first* shape comes from

```
commit #2: projection seq mismatch: expected 2, got 3  (registry.last_seq=1, journal.next_seq=4)
commit #3: projection seq mismatch: expected 2, got 4  (registry.last_seq=1, journal.next_seq=5)
commit #4: projection seq mismatch: expected 2, got 5  (registry.last_seq=1, journal.next_seq=6)
commit #5: projection seq mismatch: expected 2, got 6  (registry.last_seq=1, journal.next_seq=7)
journal contents after four 'failed' commits:
  [(1,"orient.probe"), (2,"source.scanned"), (3,"orient.probe"),
   (4,"orient.probe"), (5,"orient.probe"), (6,"orient.probe")]
open_group_len=1
```

`Core::commit` (`src/api.rs:210-212`) appends **before** it folds:

```rust
let event = self.journal.append(draft)?;
self.registry.apply(&event)?;
```

So a commit that fails contiguity has *already written its event to the
journal*, and the `?` returns before `open_group.push`. Two consequences, both
new relative to the issue text:

- **`expected` is pinned; `got` climbs by one per failed commit.** The issue's
  `expected 146, got 149` is not "three concurrent appenders" — it is **three
  prior failed commits after one un-absorbed direct write**. Between the two
  observations `expected` moved 146→156, which means something did absorb in
  between (any scan hold) and the daemon recovered, then fell over again.
- **The daemon is wedged for *all* mutations, not just the one event.** Once
  the registry is behind, every `submit`/`cancel`/`input`/`daemon.stopped`/
  `backend.probed` commit fails until some later hold happens to call
  `absorb_journaled()`. This is a stuck-daemon defect wearing a log line.

### 3b. A refutation of the issue's own premise

The issue says *"`daemon.stopped` is now simply absent for that shutdown."*
**That is not what happens.** The replay above shows every "failed" event
present in the journal at its own seq. What actually failed is the **fold and
the publish**, not the append. So:

- the journal (the authority, H1 §6) *does* contain `daemon.stopped`;
- every projection-backed read surface and every SSE subscriber does **not**;
- the operator is told `failed to journal daemon.stopped`
  (`src/daemon.rs:1485`), which is the wrong claim about the wrong layer.

This makes the defect sharper, not milder: the record and the surfaces that read
the record disagree, and the log line asserts the opposite of the truth.

## 4. Cause, located at file:line — the brief's lead is **refuted**

Every production path that holds `&mut Journal` and appends outside `commit` was
enumerated (`grep -rn "&mut Journal" src/` plus every caller of the
`record.rs` writers; all `&mut Journal` signatures live in
`src/runtime/atlas/record.rs:132,155,186,205,225,238,252` and every production
caller of them is in `src/api.rs`):

| # | Call site (pinned base) | Enclosing fn | `absorb_journaled()`? |
|---|---|---|---|
| 1 | `src/api.rs:5219` | `run_work_overlay_hook` | yes — `src/api.rs:5225` |
| 2 | `src/api.rs:5403` | `intelligence_scan` (estate-git loop) | yes — `src/api.rs:5409` |
| 3 | `src/api.rs:5475` | `intelligence_scan` (knowledge loop) | yes — `src/api.rs:5481` |
| 4 | `src/api.rs:5572` | `intelligence_scan` (external-git loop) | yes — `src/api.rs:5578` |
| 5 | **`src/api.rs:5751`** | **`intelligence_add_source`** (`src/api.rs:5693`) | **NO** |

**The cause is `src/api.rs:5749-5752`** — `POST /v1/intelligence/sources`
(routed at `src/api.rs:588-591`):

```rust
let mut core = CoreGuard::acquire(&state.core).await;
let recorded = with_atlas_write(&state, |atlas| {
    record_external_git_scan(atlas, &mut core.journal, &acquired, None)
})
```

The guard is released with no `absorb_journaled()`, leaving the registry exactly
one seq behind for a source that actually scanned (`ScanRecord::Unchanged` and
`RootUnavailable` return before the append — `src/runtime/atlas/record.rs:262-282`
— so only a genuinely new generation triggers it).

**The Atlas *scan* trigger is not the cause on this base.** All three of
`intelligence_scan`'s loops absorb, per-source, inside the loop. The brief calls
that a lead, not a verdict; the code refutes it (J3 — settled by reading the
pinned tree).

**The brief's other correction is confirmed.** `src/daemon.rs:1481` and
`src/daemon.rs:1639` both go through `core.commit(...)` under a `CoreGuard`;
`daemon.stopped` and `backend.probed` are victims of §3a's cascade, not writers.

## 5. Item 2 of the brief — did the scan front door widen this?

Answered against `335d5892`'s own tree (`git show 335d5892:src/api.rs`), which
is **not** on the pinned base (§1):

- `intelligence_scan` there spawns `run_estate_scan` (`:5553`), and that
  background task **does** absorb: `:5648`, plus a dedicated helper at
  `:5879-5889` whose own comment quotes *"every direct-journal writer must
  call this"*.
- `intelligence_add_source` there is still at `:6013` with its write at `:6071`
  and **still no absorb**.

So, plainly: **the scan front door did not introduce and did not widen #334.**
It is careful about exactly this invariant. The gap is the same single handler
before and after. What the front door *does* change is exposure — journaling now
runs off the request path — but the un-absorbed writer is a different endpoint
entirely and pre-dates it.

## 6. Boundary of this change

### In scope

1. **Close the defect at `src/api.rs:5751`** — `intelligence_add_source` folds
   what it appended before releasing the hold.
2. **Make the invariant structural, not prose.** Today it is a doc comment
   (`src/api.rs:294-296`) and nothing fails when a writer forgets. A writer that
   appends through `&mut Journal` and skips the fold must fail to compile or
   fail a guard, in the posture of
   `the_admissibility_filter_cannot_write_because_every_method_takes_an_immutable_self`.
   Proven non-vacuously: add a writer that skips it, watch it go red, capture
   the output.
3. **A deterministic regression test for the defect and its cascade** —
   the mismatch, and §3a's "one un-absorbed write wedges every later commit".
4. **A test that a shutdown always journals its own stop event** (issue item 3),
   written against what §3b establishes actually happens: the assertion is that
   `daemon.stopped` is present **and folded/published**, not merely appended.
5. **Coverage membership at birth (#231)** — any new suite wired into
   `scripts/coverage/c2-suites.sh` or `c3-spawning-suites.sh`, with
   `cargo nextest run --test c2_light -E 'test(coverage_stage_membership)'`
   run green *and* shown red against an unwired suite.

### Explicitly out of scope

- **Deciding whether a failed journal write is ever acceptable** (issue item 2 /
  brief §"the one J0"). Escalated, not chosen — see §7.
- **Any retry/serialisation redesign of the append path.** That is item 2's
  implementation and cannot start before item 2 is ruled.
- **Changing `Core::commit`'s append-before-fold order** to make a failed commit
  not append. That changes durable-record semantics and belongs to the same J0.
- **The scan front door (`335d5892`) and any merge of it.** Not on the pin (§1),
  not this run's tree, and §5 shows it is not implicated.
- **Correcting the misleading `failed to journal …` log wording** at
  `src/daemon.rs:1485` / `:1647` — operator-visible text about record loss, which
  is the visibility half of item 2's J0. Recorded here so it is not lost.
- **Estate-drift, Atlas schema, ATTACH, or any second Atlas writer.** Standing
  constraint: one Atlas database, the daemon stays sole writer (A1 §12).
- **Anything clock-shaped.** No deadline loop, no `Instant::now() + BUDGET`, no
  wall-clock or ratio assertion (stripped in `bdda34f3`).
- **Backfilling or re-folding journals already wedged on a live install.** A
  recovery/`sgt doctor` path is a separate change with its own acceptance.

## 7. The one J0 — escalated, with the evidence this stage gathered

**Question (issue item 2):** is a failed journal write ever acceptable?

**What this stage found that bears on it, and that the issue did not know:**

- A failed `commit` **still appends** (§3a). So "a failed journal write" is not
  actually the failure mode in #334 — the write lands and the *fold and publish*
  are lost. The contract question is therefore sharper than it was posed: it is
  about whether the journal and the surfaces that read it may ever disagree.
- The failure is **cascading and self-sustaining** (§3a): one un-absorbed write
  wedges every subsequent mutation until an unrelated hold absorbs. Any "named
  acceptable window" would have to be a window that closes itself; today none
  does.
- The loss is **invisible where an operator looks** — a `WARN`/`ERROR` line
  (`src/daemon.rs:1485`, `:1647`) that also *misdescribes what was lost* (§3b).

**Recommendation, offered not taken:** never acceptable, and closed
structurally rather than by retry — the invariant is already expressible as
"the hold folds what it appended", and §6 item 2 makes that unforgettable.
`Core::commit`'s append-before-fold ordering should additionally be revisited so
a failed fold cannot leave an orphan in the journal, and the log wording should
be corrected to say what was actually lost. **All three are J0 and none is
decided here.**

Items 1, 3, 4 and 5 of the brief proceed without this answer, exactly as the
brief states.

## 8. Rungs cited

- **J5** — the contract (A1-01, A1 §12, H1 §6) over the brief, where they
  differ; the pinned revision over a prompt's claim about what is merged (§1).
- **J3** — the cause (§4) and the scan-front-door answer (§5) are settled by
  reading the pinned tree and `335d5892`'s tree; no judgment was needed.
- **J2** — this stage's delegated phrasing of the boundary (§6) and of which
  parts of the brief are this change versus adjacent work.
- **J0** — item 2 (§7), escalated.
- **R2** — the repair mechanism already exists (`Core::absorb_journaled`,
  `src/api.rs:296`); nothing new is built to close §6 item 1. The structural
  guard (§6 item 2) is `10-implement`'s rung to cite.
