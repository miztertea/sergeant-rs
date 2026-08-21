# ADR 0019: Bounded retention — the knob, the mechanism, and its tradeoffs

**Status:** Accepted, 2026-08-21.

## Context

`docs/adr/0003-durability-promise-and-storage-preconditions.md`'s D7
amendment (2026-08-21) states *what* the durability promise now means once
history can be legitimately retired: full trajectory while retained,
never forever, never silently. This ADR records *how* — the mechanism
issue #17's Q1–Q10 rulings record (2026-08-21) ruled into existence,
replacing the compressed-archive design `docs/proposals/journal-archival-rule-c.md`
had proposed (that document's disposition header points here).

## Decision

**Count-based retention, per estate, declared in `sergeant.toml`.**
`[estate] retention = N` (`src/domain/estate.rs`'s `DEFAULT_RETENTION`/
`MIN_RETENTION`) names how many terminal Works of history this estate
keeps; absent, it defaults to **1000**. A prune deletes whole journal
segments, oldest first, once every Work they hold is terminal, retired
whole (`src/runtime/prune.rs::retired_whole` — broader than eviction's own
`run_is_settled`: a `Failed` Work is retired whole too, or one old failure
would pin the journal forever), and past the newest `N` by `last_seq`. Every
prune is journaled (`prune.intent` → destructive acts → `prune.completed`)
and automatic — the declared policy is the whole authorization surface,
with no override of any kind (I-W3-11).

**The default is 1000, on a measured basis, not a guess.** This estate's
own live dogfood journal measured, 2026-08-21: 60 real Works = 19,184
events / 16.7 MB journal + 92 MB blobs ≈ **1.8 MB per real Work**. Retention
1000 ≈ 1.8 GB bounded total; 10,000 ≈ 18 GB — a legitimate per-estate
setting, rejected as the default. `MIN_RETENTION = 64` is not a
correctness bound (the prune predicate is sound at any N — a non-terminal
or unsettled Work is never prunable regardless of the cap); it exists
because a value below the daemon's own in-memory terminal-cache capacities
(`TERMINAL_RUN_CACHE_CAPACITY = 512`, `TERMINAL_WORK_CACHE_CAPACITY = 1024`)
retains less history than the process already holds live in memory — a
knob set under its own working set, refused by name at manifest-parse
time rather than discovered later.

**Retention is eventually-exact, by segment, not by Work.** Works'
events interleave within a segment file, and only whole segments are ever
unlinked (I-W3-4). A Work can briefly outlive the cap by sharing a segment
with a younger one; `retention` is a floor on what is kept, never a
ceiling. This is the direct cost of the atom being the *segment* for
deletion purposes while the atom is the *Work* for the durability
guarantee (D7 point 2) — the two atoms are different, and the gap between
them is this tradeoff.

**The command ledger is exempt, by journaled residue, not by cache.** §26's
exact-once command idempotency must survive a prune, or pruning would
reopen the duplicate-Work-creation hole retention exists to bound safely
around. Every pruned command's ledger key (`command_id → status-class,
work_id for submits`) is carried in the same journaled residue that
survives the Work rows themselves (`PrunedCommandRow`, `w3-spec.md` §2.3) —
durable in the journal, not merely cached, so a retried `command_id` below
the floor is refused by name even after the cache that would normally
answer it has been deleted and rebuilt from scratch.

**A crash mid-prune completes at the next daemon start, evidence-based.**
The same shape `src/runtime/recovery.rs`'s interrupted-teardown completion
already uses: a `prune.intent` with no matching `prune.completed` is
finished from the intent's own recorded targets, never re-planned, never
started from suspicion (`recovery.rs:118-123`'s refusal boundary is cited,
not re-litigated). This is the one bounded exception to "no automatic
deletion" the rulings record accepts (Q9).

**`FloorState` bumped to schema v2** (`sergeant.floor-state.v2`) to carry
`first_seq` per retained Work (without which pruning is inert on a
cache-hit start — the no-straddle predicate has nothing to check a seeded
Work's start against) and the residue sections
(`pruned_works`/`pruned_commands`/`pending_prune`/`quarantined_blobs`). An
old v1 cache is a named miss (`"miss:unknown_schema"`), not an error — one
extra full floor replay on the first 0.1.3 start, the license the
forward-compatibility rule (`w2-spec.md` §2.7 rule 3) grants a schema bump
precisely for this reason.

**Deferred, on purpose: a manual `sgt journal prune` verb.** Three reasons,
weighted in order (`w3-spec.md` §12):

1. Pruning must run inside the daemon process — it appends through `Core`
   under the group-commit guard and unlinks under the journal's own
   exclusive lock, both of which only the running daemon holds for the
   life of the process. A CLI-invoked verb is not "call the engine," it is
   a new authenticated endpoint, client method, and request/response
   shape — explicitly this wave's surface work, not a free add-on to W3's.
2. There is no `sgt journal` command group to hang it on yet — the verb
   costs a new top-level group and its own `--dry-run`/`--yes` grammar.
3. The declared policy is already the whole authorization (A1); a manual
   trigger adds no new capability, only a way to ask for one that already
   runs automatically at start and at rotation. It is pure ceremony, which
   is exactly why A6 named it as the safe cut line if the implementation
   night ran short.

What ships instead: `prune::stall_report` (read by `sgt doctor`'s
`journal_growth` row, this ADR's own mechanism above) and `--rebuild-cache`
(W2) cover the two things a manual verb would actually be for — seeing
what is or is not happening, and forcing the cache to catch up — without
adding a trigger the policy does not already provide.

**Deferred escalation, named and not built: a persisted blob-refcount
cache.** The mark-and-sweep blob liveness scan (`src/runtime/prune.rs` §5.1)
reads the whole retained journal once per prune cycle, off the mutation
guard, to find which blobs survive — O(retained journal) per cycle,
roughly once per journal turnover after `PRUNE_BATCH_MIN_SEGMENTS`
batching. A persisted blob-refcount index, folded by the same shared
startup pass and carried in the cache, would make marking O(condemned)
instead — strictly faster — but it is a new durable derived structure with
its own crash-consistency story, landed on top of a wave that already
introduced the codebase's first deletion engine. It is named here as the
escalation path if the dogfood measurement (`tests/w3_prune_measurement.rs`)
ever shows the scan is the bottleneck; it is not built, and nothing in
0.1.3 depends on it existing.

## Alternatives considered

**The original archival design** (`docs/proposals/journal-archival-rule-c.md`):
a compressed `archive/` tier with per-segment manifests and a
`sgt journal archive` preview ceremony, retaining history forever in
degraded (compressed, non-replayable) form rather than deleting it. Ruled
out at the #17 grilling session (premise amendment A2): once pruning is
legal at all, the compressed middle layer loses its reason to exist — it
solved "keep everything, just smaller," and the ruling replaced the
question itself with "keep bounded history, precisely, or not at all."

**A blob refcount escalation, built now rather than named.** Rejected for
this release for the reason stated above — a second durable derived
structure's crash-consistency story is real design work, and this wave's
budget went to landing the deletion engine correctly rather than to
pre-optimizing its slowest internal pass.

**A `--retention` CLI override / environment variable.** Rejected
structurally, not merely by default (I-W3-11): Q7's ruling that stalling is
reported, never overridden, only holds if there is no lever anywhere —
runtime, environment, or API — that could waive it. `tests/w3_prune_engine.rs::no_configuration_or_flag_can_lower_the_prune_predicate`
is the source-scan enforcement.

## Consequences

Disk stays bounded (≈1.8 GB at the default 1000, on this estate's measured
basis) and startup stays fast (rung 0 + the 16-segment/128 MiB window,
`w2-spec.md`) regardless of how long an estate has been running. The
accepted costs: eventually-exact retention (a Work can briefly outlive the
cap), an unretryable `Failed` Work past the cap, unbounded (if very slow-
growing) residue accumulation in the journal itself, an O(retained
journal) blob mark scan once per turnover, and no manual trigger verb in
this release. Every one of these is a deliberate, named tradeoff, not an
oversight — each is cited above and in `w3-spec.md`'s own §3/§14 for the
full argument.

## Open questions

- Whether chronic prune stalling (a routine, not exceptional, blocking
  Work) in practice warrants the narrower predicate Q7 names as the
  escalation path, is not yet measured — `sgt doctor`'s `journal_growth`
  row is what would surface that evidence over time.
- The blob-refcount escalation's actual payoff is unmeasured; nothing in
  0.1.3 depends on measuring it, but a future wave revisiting prune
  performance should start from `tests/w3_prune_measurement.rs`'s
  read-only dogfood harness rather than re-deriving a cost model from
  scratch.
