# 40-classify: apply the ICM decomposition ladder

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/icm-ladder.md | L3 | the decomposition ladder (§6.1-6.7) this stage applies to every unit, in order |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| references/classification-record-shape.md | L3 | the exact record fields, conditional requirements, and the empty-`alternatives_considered` rule |
| ../30-normalize/output/behavior-units.normalized.ndjson | L4 | upstream artifact produced by `30-normalize` — the normalized behavior units this stage classifies |

## Purpose

Apply `../_config/icm-ladder.md`'s decomposition ladder to every normalized
behavior unit, one classification record per unit, per
`references/classification-record-shape.md`. This stage assigns
representation; it does not draft, cluster, or materialize anything —
that is `50-synthesize`'s and `60-draft`'s job.

## What must become true here (durable outcome)

`output/classifications.ndjson` exists, one record per line, with every unit
from `../30-normalize/output/behavior-units.normalized.ndjson` classified
exactly once (no unit skipped, no unit classified twice). Every record
carries `rationale` and `alternatives_considered` meeting
`references/classification-record-shape.md`'s bar — not generic, not
copy-pasteable onto an adjacent rung.

## How to do it

0. If `../30-normalize/output/behavior-units.normalized.ndjson` opens with
   `# AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

Go through `../30-normalize/output/behavior-units.normalized.ndjson` in
order, one unit at a time:

1. Ask `../_config/icm-ladder.md`'s questions **in order**, 6.1 through 6.7,
   and stop at the first one that answers yes. This ordering is itself the
   conservatism rule — do not skip ahead to a rung that "feels right" before
   ruling out every lower one.
2. For a `stage` classification (6.3), explicitly apply the reimplementation
   test and say so in `rationale`: if this behavior's current mechanism were
   replaced tomorrow, would the checkpoint still exist? If the honest answer
   trends toward "no, this is just a script," reclassify as `helper` (6.5)
   instead.
3. For an `engine-gap` classification (6.7), this is the last rung — reached
   only after 6.1–6.6 have each been tried and failed for a rung-specific
   reason. Write the full six-field template from
   `../_config/icm-ladder.md`'s §6.7 section into `engine_gap`, verbatim
   field names, all six populated. If any field would read "would be
   convenient" or "could be more elegant" in any form, this is not an
   engine-gap unit — reclassify to the best-fitting lower rung instead and
   let the temptation itself go unrecorded (the classification record has
   no field for it; it is not evidence of anything).
4. For an `obsolete-mechanism` disposition, name the specific settled fact
   (a deviation-register ruling, an already-documented invariant) that
   already replaces the mechanism — carried from the unit's `notes` field.
   If you cannot name one, this is not `obsolete-mechanism`; classify by
   whatever surviving policy the behavior actually states.
5. Write the record per `references/classification-record-shape.md`. Do not
   copy the unit's evidence fields into it — only `behavior_id` refers back.
6. Move to the next unit only once its classification is written.

Process the entire input file. A classification you are genuinely torn on
between two adjacent rungs is still resolved to one — record both rungs in
`alternatives_considered` and let `rationale` state the tension honestly;
`80-adversarial-review` exists to challenge exactly this kind of call.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
