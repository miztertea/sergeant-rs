# Partition checkpoint & retry protocol

Layer 3 (stable across runs), local to `20-harvest`. States how this stage
crosses more than one actor turn/attempt when `10-inventory`'s partition set
does not fit in one turn, without an engine change and without losing or
duplicating coverage. Written against GP-1's adjudicated disposition
(`docs/gauntlet/runs/n2-run2/grammar-pressure-report.md`) — see
`../../CONTEXT.md`'s "v2: how `20-harvest` handles volume" for the choice
this protocol implements and the alternative it rejects.

## Why this works without a new engine capability

`output/` is Git-tracked on this run's Work branch and persists across a
stage retry (`sgt work retry` re-enters a stage as a fresh execution — fresh
actor turn, fresh context window — against the artifacts already on disk;
`docs/gauntlet/runs/n2-run2/grammar-pressure-report.md` GP-1 R1 measured
this working in run 2's own journal). So a stage that stops partway through
its work, having durably recorded exactly how far it got, can be resumed by
a **later, independent attempt of the same stage** picking up where the
last one left off — no fan-out, no dynamic stage creation, no
actor-initiated mid-turn pause (that primitive is GP-2's confirmed but
not-yet-wired gap; this protocol does not depend on it).

## The ledger

`output/partition-ledger.md` is this stage's own durable checkpoint record,
`promote`d like every other artifact here (`output/README.md`). One row per
named partition from `../10-inventory/output/inventory.md`, in the order
recorded there:

```text
| Partition | Status | Unit id range | Notes |
|---|---|---|---|
| root-level agent instructions | done | BU-0001–BU-0023 | |
| bin: fleet dispatch & lifecycle (dispatch) | done | BU-0024–BU-0041 | |
| bin: fleet dispatch & lifecycle (recovery) | pending | | not reached this attempt |
```

`Status` is exactly `done` or `pending` — never a third state. A partition
is `done` only once every file in it has been read, has produced its
behavior units (or an explicit zero-units decision, per `../CONTEXT.md`),
**and** has been swept per
`references/consequence-class-checklist.md`.

## On stage entry (every attempt, first or Nth)

0. If `../10-inventory/output/inventory.md` opens with
   `# AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2 (unchanged from before this protocol
   existed).
1. Check whether `output/partition-ledger.md` already exists.
   - **It does not exist** (this is the first attempt): create it, seeded
     with every named partition from `../10-inventory/output/inventory.md`
     in its recorded order, every row `pending`.
   - **It already exists** (this is a retry of a prior attempt): read it as
     the authoritative record of what is already done. Do not re-read or
     re-extract a `done` partition's files, and do not re-run the
     consequence-class sweep on a partition already marked `done` — the
     ledger, not memory of a previous conversation (which this fresh
     execution does not have), is what makes the earlier work trustworthy
     to skip.

## Working through partitions

Process `pending` partitions in ledger order. For each partition:

1. Read every file in it.
2. Extract behavior units per `../CONTEXT.md`'s ordinary method (one unit
   per independently-triggerable behavior, cited per
   `../_config/evidence-policy.md`).
3. Apply `references/consequence-class-checklist.md` to every file in the
   partition and record one row per file in
   `output/consequence-class-sweep.md`.
4. **Append** this partition's behavior units to `output/behavior-units.ndjson`
   now, before moving to the next partition — do not hold units in memory
   across partitions to write once at the end. A turn that ends partway
   through the *next* partition must not lose units already captured for
   partitions before it.
5. **Append** this partition's sweep rows to `output/consequence-class-sweep.md`
   now, for the same reason.
6. Mark the partition `done` in `output/partition-ledger.md`, with its unit
   id range and (if `references/consequence-class-checklist.md` found
   nothing in one or more classes for one or more files) a one-line note —
   not silence.

## Stopping honestly when a turn will not fit everything

If, partway through, this attempt will not finish every `pending`
partition:

- **Stop at a partition boundary, never mid-partition.** A partition is
  only ever `done` or `pending` in the ledger — there is no `half-done`.
  Finish the partition you are on (steps 1–6 above) before stopping; do not
  shortcut its extraction depth or its consequence-class sweep just to
  "finish" it under time pressure — an incompletely-swept partition marked
  `done` is worse than an honestly `pending` one, because nothing downstream
  will know to re-check it.
- End the turn normally — an ordinary stage completion, the same as any
  other stage that finishes its declared work. This is not a fabricated
  signal; it is this stage genuinely not yet meeting its durable outcome
  (`../CONTEXT.md`: every partition must reach `done`), recorded plainly by
  the ledger rather than papered over.
- Do **not** invent an ad hoc sub-procedure, and do not silently truncate
  coverage by skipping straight to the *last* partition to make the ledger
  look more complete than the actual coverage achieved — partitions are
  worked in the declared order for a reason: a reviewer scanning the ledger
  can tell real progress from a shortcut.
- A ledger with any `pending` row at the end of an attempt is real signal —
  someone (a human operator, or an orchestrating caller of this Work) needs
  to notice it and cause another attempt of this stage. `sgt work retry` is
  **not** that mechanism (fixes #53; measured at N2 run 4, 2026-08-11,
  `docs/gauntlet/notes/n2-fake-backend-semantics.md`): retry is only legal
  against a failed/blocked/waiting stage, and this stage is neither —
  issuing it against a held or freshly-ended stage is refused (409 under
  the fake-held harness) or has no state left to act on (under a real
  backend). The two resume paths actually measured to work are:
  - **Same-hold continuation** (measured under the fake-held harness): the
    stage's `needs_input` hold persists across external actor attempts with
    no engine command needed in between — each attempt re-reads this ledger
    per "On stage entry" above, and the final attempt's single `respond`
    (not `retry`) is what advances the stage past the hold.
  - **Cross-run reseed** (measured under a real backend): a harvest turn
    that ends with `pending` rows simply ends its turn — with BS2's settle
    fix the stage completes or blocks and the workflow moves on. There is
    no mid-protocol engine action to take; resumption is a **later,
    independent attempt of this stage** (a fresh invocation of this
    workflow, or an orchestrating caller re-entering it), which reads this
    same ledger under "On stage entry" above and continues from the first
    `pending` row exactly as a same-hold continuation would.

  Either way, this stage — like `00-contract`'s fail-closed marker — has no
  way to trigger its own continuation: record the incomplete ledger
  honestly and stop, exactly as `00-contract` writes
  `# AMBIGUOUS — NOT RESOLVED` and stops rather than guessing.

## Completion

The durable outcome in `../CONTEXT.md` is met once every row in
`output/partition-ledger.md` is `done`. At that point
`output/behavior-units.ndjson` and `output/consequence-class-sweep.md` are
each a single, complete, append-accumulated file — indistinguishable in
shape from what one long turn would have produced, whatever number of
attempts it actually took. No downstream stage's `CONTEXT.md` needs to know
or care how many attempts `20-harvest` used.
