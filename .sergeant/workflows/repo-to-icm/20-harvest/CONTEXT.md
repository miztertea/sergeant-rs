# 20-harvest: extract source-cited behavior units

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-contract/output/contract.md | L4 | upstream artifact produced by `00-contract` — subject repository and pinned revision every citation resolves against |
| ../10-inventory/output/inventory.md | L4 | upstream artifact produced by `10-inventory` — which files are dispositioned `decompose`, and their partitions |
| ../_config/evidence-policy.md | L3 | the required record shape and the quote+hash discipline |
| ../_config/run-discipline.md | L3 | the blindness rule (this stage mints citations — the highest-risk stage for it) and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| references/extraction-example.md | L3 | a worked example of the shape, including the one-behavior-per-unit split |
| references/partition-checkpoint-protocol.md | L3 | how this stage crosses more than one attempt when partitions do not fit in one turn — read this before "How to do it" below |
| references/consequence-class-checklist.md | L3 | the mandatory five-class sweep every `decompose` file gets, in addition to ordinary extraction |

## Purpose

Extract source-cited behavior units from every `decompose`-dispositioned
file in the inventory. **Do not assign an ICM representation, workflow, or
stage to any unit here** — that is a later stage's contract. This stage's
job is: state the behavior, cite where it came from, verify the citation —
and, for every file, deliberately sweep it for the five consequence classes
`references/consequence-class-checklist.md` names, not just whatever
behavior happens to stand out on a first read.

## What must become true here (durable outcome)

Three things, together:

1. `output/behavior-units.ndjson` — one JSON record per line, in the exact
   field shape `../_config/evidence-policy.md` defines.
2. `output/partition-ledger.md` — every named partition from `10-inventory`
   marked `done`, per `references/partition-checkpoint-protocol.md`. A
   partition can legitimately produce zero behavior units once its files
   are read closely (record why), but it cannot be marked `done` without
   having been read and swept.
3. `output/consequence-class-sweep.md` — one row per `decompose` file, per
   `references/consequence-class-checklist.md`, with no blank cells.

The durable outcome is not met while `partition-ledger.md` has any
`pending` row — that is real, honestly-recorded incomplete coverage, not a
defect to hide (see the protocol file for what to do about it: stop at a
partition boundary, do not force through remaining partitions).

## How to do it

0. If `contract.md` or `inventory.md` opens with `# AMBIGUOUS — NOT
   RESOLVED`, do not proceed — follow `../_config/run-discipline.md` §2.

Follow `references/partition-checkpoint-protocol.md` end to end — it covers
reading (or creating) `output/partition-ledger.md`, working through
partitions in order, and stopping honestly at a partition boundary if a
turn will not finish everything. Within each file, once you are actually
reading it:

1. Read it. Identify each independently-triggerable behavior it states or
   implies — not one unit per file, not one unit per paragraph, but one
   unit per behavior (`../_config/evidence-policy.md`, "One behavior per
   unit").
2. For each behavior, write the record: `statement`, `source` (path,
   locator, quote, quote_hash — computed against the real bytes, not
   estimated), `scope`, `trigger`, `outcome`, `authority`, `confidence`, and
   `notes` if there's mechanism/intent separation to record.
3. Assign `id`s sequentially and keep them stable once written — a later
   stage or a reviewer may cite them.
4. **Apply `references/consequence-class-checklist.md`'s five hunt
   questions to this same file** — safety, identity, recovery, delivery,
   human-decision — and add a row to `output/consequence-class-sweep.md`
   (per-partition append, per the protocol file). This is not optional and
   not deferred to "if time allows": a file is not done until both its
   ordinary extraction and its consequence-class sweep are recorded, even
   if the sweep finds nothing (record `swept, none found`, not silence).
5. Move to the next file only once the current one's behaviors are fully
   captured (or consciously judged zero, with a stated reason) *and* its
   sweep row is written.

**Do not manufacture units to fit an expected shape.** A corpus where every
unit suspiciously lines up with some workflow boundary you can already
imagine is grounds for a reviewer to doubt whether extraction actually
happened before classification did (`docs/icm/record-shapes.md` §3 rule 2).
Extract what the source says: no more, no less, and not reshaped to be
convenient later.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifacts and their dispositions.
