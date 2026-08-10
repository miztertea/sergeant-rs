# 20-harvest: extract source-cited behavior units

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-contract/output/README.md | L4 | upstream artifact produced by `00-contract` — subject repository and pinned revision every citation resolves against |
| ../10-inventory/output/README.md | L4 | upstream artifact produced by `10-inventory` — which files are dispositioned `decompose`, and their partitions |
| ../_config/evidence-policy.md | L3 | the required record shape and the quote+hash discipline |
| references/extraction-example.md | L3 | a worked example of the shape, including the one-behavior-per-unit split |

## Purpose

Extract source-cited behavior units from every `decompose`-dispositioned
file in the inventory. **Do not assign an ICM representation, workflow, or
stage to any unit here** — that is a later stage's contract. This stage's
only job is: state the behavior, cite where it came from, verify the
citation.

## What must become true here (durable outcome)

`output/behavior-units.ndjson` exists, one JSON record per line, in the
exact field shape `../_config/evidence-policy.md` defines — and every
`decompose`-dispositioned file or partition from `10-inventory` has been
read and has produced at least one unit, or is recorded as producing zero
units with a stated reason (a file can genuinely turn out to carry no
independent behavior once read closely; that is a legitimate outcome, but
it must be a recorded decision, not a silent gap).

## How to do it

Work through the inventory's `decompose` partitions in the order
`10-inventory` recorded them. For each file in a partition:

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
4. Move to the next file only once the current one's behaviors are fully
   captured or you have consciously decided it contributes zero units (and
   recorded why, either inline as a `notes`-only skip line or in a short
   coverage note at the end of the output file — pick one convention and
   hold it for the whole run).

**On volume.** This stage runs as one actor turn per this workflow's
current grammar (proposal §9.2) — there is no fan-out to parallel harvest
sub-agents per partition, even though a large inventory may make that
tempting. Work through partitions sequentially within the turn. If the
volume in scope genuinely cannot be covered completely within one turn,
do not silently truncate coverage and do not invent an ad hoc sub-procedure
to work around it — finish as much as the turn allows in inventory order,
and record plainly, in the output file or its accompanying notes, exactly
which partitions were not reached. That gap is real signal for this
workflow's grammar-pressure report, not a defect to hide.

**Do not manufacture units to fit an expected shape.** A corpus where every
unit suspiciously lines up with some workflow boundary you can already
imagine is grounds for a reviewer to doubt whether extraction actually
happened before classification did (`docs/icm/record-shapes.md` §3 rule 2).
Extract what the source says: no more, no less, and not reshaped to be
convenient later.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
