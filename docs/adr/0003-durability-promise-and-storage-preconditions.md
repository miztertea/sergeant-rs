# ADR 0003: Durability promise and storage preconditions

**Status:** Accepted, 2026-08-14.

## Context

`docs/DEVELOPMENT.md`'s architecture invariants already state that "the
journal is the only truth" and that "work state ≠ process state": a Claude
"session" is a durable conversation identity, the OS process exists only
per turn, and restart reconciliation (`src/runtime/recovery.rs`) resumes
only on unambiguous evidence, failing closed into `blocked` on ambiguity
rather than guessing. Bringing WSL2 into the measured-target set (ADR 0001)
raises a question the interview had to settle explicitly: what does this
architecture actually promise when the machine underneath it goes away,
and does WSL2's specific way of going away — a distro teardown — threaten
that promise any differently than sleep, shutdown, or a kernel panic do.

## Decision

**The promise is durability, not daemon survival (D5).** Sergeant makes no
claim about the user's machine staying up. Sleep, shutdown, a WSL2 distro
teardown, and a kernel panic are all the same class of event from the
architecture's point of view, and none of them are sergeant's to control.
What the architecture actually promises is that the journal makes work
**resumable** — resume the work, not merely continue a session. This is a
direct consequence of the invariants `docs/DEVELOPMENT.md` already states:
everything but the journal is a disposable projection rebuilt from it, and
restart reconciliation's whole job is turning "the daemon came back after
an unknown gap" into either a resumed work item or an honestly-labeled
`blocked` one. The platform-relevant probe this decision implies is
therefore not "does the daemon survive" — nothing is supposed to guarantee
that — but "after an abrupt daemon death, does work actually resume, or
does it land in `blocked`?"

That probe surfaced a known degradation on macOS at the time of this
interview: restart reconciliation gathered its liveness evidence from
`/proc` alone (`src/backend/claude.rs`'s `session_liveness_excluding`), and
with no `/proc` on macOS, every liveness check returned
`Liveness::Unknowable`, so per the fail-closed contract restart
reconciliation already applies ("ambiguous states fail closed to blocked
with evidence", `src/runtime/recovery.rs:76-77`), every *ambiguous* restart
on macOS failed closed into `blocked` rather than resuming. ADR 0002 has
since moved this fact behind the platform boundary
(`crate::platform::process::running_processes`,
`session_liveness_excluding` at `src/backend/claude.rs:553-559`) and added a
`ps`-based macOS arm, so a macOS restart is no longer *unconditionally*
`Unknowable` — but that arm is marked **UNVERIFIED** (never run on a real
macOS host), so the gap this paragraph records is narrowed, not closed.
This is tracked as **issue #18** — confirmed live in this repo as "/proc
portability" (`GAUNTLET.md` line 1787, "P0 close-out... /proc portability
#18"; `docs/gauntlet/contracts/N0.md`, "R-N0-5 — #18 (/proc portability):
deferred to N5"). It remains open until measured on a real macOS host, per
ADR 0002's own UNVERIFIED marking.

The interview record also corrects itself explicitly here: an earlier
framing offered by the orchestrating session during the interview, that a
WSL2 distro teardown "breaks the core promise," was **wrong**, and the
owner corrected it. A distro teardown is an ordinary machine-went-away
event, exactly like sleep or shutdown, and durability as defined above
already covers it — it is not a special case requiring new machinery.

**Unsuitable filesystem is a hard failure, not a warning (D6).** If the
data dir sits on a filesystem where advisory locking is unreliable,
`sgt init` and daemon start must refuse outright, and `sgt doctor` must
carry a named-remedy row for it — consistent with this repo's existing
convention that a missing capability surfaces as `sgt doctor`'s named
remedy, never a silent skip (`AGENTS.md`'s Guardrails section). The
architecture rests on exactly one process holding `daemon.lock`
(`docs/DEVELOPMENT.md`'s "One owner" invariant; `src/daemon.rs:49-50`,
`111`; the advisory-lock take in `src/runtime/fsutil.rs:24-29`) — a lock
that silently does not actually hold is worse than a refusal at `init`
time, because it fails in a way nobody notices until two daemons are
racing the same data dir. The predicate this decision generalizes to is
"advisory locking unreliable," deliberately not "the path is under
`/mnt/c`" — the general predicate is what makes it also cover NFS and SMB
shares, which have the identical `flock` problem for an unrelated reason.
The originating, concrete case is WSL2's `drvfs`: an estate cloned under
`/mnt/c/...` crosses the Windows filesystem boundary, where `flock` is
unreliable and git worktree operations, file watching, and general
performance all degrade. This is detectable from `/proc/mounts` on Linux
(including inside a WSL2 distro); the macOS detection path is unmeasured
and is left as an open question below rather than assumed.

## Alternatives considered

No alternative to D5's durability framing was on the table in the
interview beyond the rejected "WSL teardown breaks the promise" framing
recorded above as a correction, not a considered-and-rejected alternative
in its own right.

No alternative to D6's hard-failure posture (such as a warning-only
`sgt doctor` row, or detecting only the originating `/mnt/c` case by name)
is recorded in the interview beyond the generalization from "`/mnt/c`
specifically" to "advisory locking unreliable" described above as the
decision itself, made precisely so NFS and SMB are covered by the same
mechanism rather than needing their own special-cased detection later.

## Consequences

D5 is mostly a clarifying, non-code decision, but it fixes what "measured"
in ADR 0001 has to actually mean for durability: a host isn't measured
durable by observing that the daemon didn't crash, it's measured durable
by observing that a killed daemon's work actually resumes. The known
macOS liveness gap (#18) — narrowed by ADR 0002's UNVERIFIED `ps`-based arm
but not yet measured on a real host — is a real, already-tracked cost of
that standard, not a new one introduced here.

D6 implies work that **does not exist yet** in this codebase — there is no
current check for advisory-locking-unreliable filesystems in `sgt init` or
daemon start (`grep` for `drvfs`, an unreliable-filesystem check, NFS, or
SMB handling in `src/cli.rs`/`src/daemon.rs` turns up nothing). This ADR
records the decision; it does not implement the refusal, the `sgt doctor`
remedy row, or the `/proc/mounts` detection, all of which are separate,
not-yet-filed implementation work.

## Amendment (2026-08-21): the durability promise under bounded retention

**D7 — durability while retained is total; durability forever is not the
promise.** Issue #17's Q1–Q10 rulings record (2026-08-21) ruled bounded,
count-based retention into existence: `[estate] retention` names how many
terminal Works' worth of journal history this estate keeps, and older
history — segments, and the blobs only their Works referenced — is deleted,
under policy, journaled. D5 above already established that durability means
*resumable*, not *the daemon survives forever*; D7 is the same discipline
applied to disk rather than to process lifetime, and it answers the
question D5 did not: what does "resumable" mean once some history is gone
by design rather than by accident.

The promise, precisely:

1. **While a Work's history is retained, its full trajectory is
   replayable — never a silent partial view.** A client reading a retained
   Work's transcript, graph, or event history gets everything that Work
   ever did, not a window into it. Nothing in this codebase serves a
   retained Work's history with an undisclosed gap.
2. **The Work is the atom of retention.** It is retired whole or not at
   all — never bisected by a segment boundary, a time cut, or a partial
   unlink. `src/runtime/prune.rs`'s horizon predicate (`I-W3-1`, the
   no-straddle invariant) is this decision made mechanical: a prune may
   never leave some of a Work's events and delete the rest.
3. **Retention-for-all-time is explicitly not promised.** Disk is finite
   and startup must stay fast (D5's own "the architecture rests on the
   journal, and the journal must stay fast to read" concern, extended);
   retiring old, already-terminal history under a declared policy is a
   legitimate operation, not a degradation of the durability promise. A
   `Failed` Work older than the newest `retention` becomes unretryable —
   the retry edge needs history that policy has since retired — and that
   is this decision working as intended, not a defect (`docs/adr/0019-*.md`
   states the tradeoff in full).
4. **The crime is silent deletion, never deletion itself.** Every discard
   flows from one of exactly two sources — a declared policy in
   `sergeant.toml`, or (for the surface-teardown case D5/D6 already cover)
   explicit operator action — and leaves a journaled record naming what
   went and under which authority. `sgt doctor`'s `journal_growth` check and
   `work show`'s pruned-by-name answer exist specifically so that record is
   never buried where an operator cannot find it (`docs/adr/0019-*.md`, §16
   of `w3-spec.md`'s design).

This does not touch D5 (durability ≠ daemon survival) or D6 (unsuitable
filesystem is a hard failure) — both stand exactly as decided. D7 is a
sibling decision about a question this ADR simply had not yet been asked.
The retention mechanism itself — the knob, its default, the prune protocol,
its measured basis, and every tradeoff it accepts — is `docs/adr/0019-bounded-retention.md`'s
job, not this one's; D7 states only what the durability promise now means
in light of that mechanism existing.

**Alternatives considered.** Treating any pruning as a violation of D5
(rejected by the owner during the #17 grilling session, per the rulings
record's premise amendment A1 itself: "the promise becomes... retention-for-
all-time is explicitly not promised"). Leaving the durability promise
unamended and treating retention as an unrelated, undocumented exception
(rejected: it would leave exactly the "is a discard ever silent" question
this ADR exists to answer unanswered for the one case that most needs it).

**Consequences.** A `Failed` Work past the cap is a real, accepted
degradation (point 3): `sgt work retry` on one answers `work_not_found` or
the pruned-by-name shape, not a special error naming the tradeoff — this is
recorded here so a future reader does not mistake it for a bug. The
residue that makes point 4 possible (`pruned_works`, `pruned_commands`)
grows without bound in the journal itself, at roughly 0.016% of a Work's
own footprint (measured, `w3-spec.md` §6.4) — a cost paid once, in
exchange for never re-opening a silent-404 hole.

## Open questions

- The macOS detection path for advisory-locking-unreliable filesystems is
  explicitly unmeasured. `/proc/mounts` is a Linux-only mechanism (and per
  D5's own `/proc`-portability gap, `/proc` itself is already known absent
  on macOS); what macOS uses instead (`mount`, `statfs`, `diskutil`, or
  something else) was not decided in the interview and needs its own
  measurement pass before D6 can be implemented on that target.
- D6 does not specify what "advisory locking unreliable" is detected
  against beyond the `/mnt/c`/drvfs originating case and the NFS/SMB
  generalization named explicitly — whether other network or overlay
  filesystem types need their own entries in that predicate is left open.
