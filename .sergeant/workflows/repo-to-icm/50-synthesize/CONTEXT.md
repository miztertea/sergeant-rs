# 50-synthesize: cluster classified units into candidates

## Inputs

| File | Layer | Why |
|---|---|---|
| references/synthesis-method.md | L3 | the six buckets and how to cluster/order/name candidates within each |
| ../40-classify/output/README.md | L4 | upstream artifact produced by `40-classify` — the per-unit classification records this stage clusters |
| ../30-normalize/output/README.md | L4 | upstream artifact produced by `30-normalize` — the normalized `statement`/`trigger`/`outcome` text needed to name candidates and order stages (classification records alone carry no readable behavior text) |

## Purpose

Turn the flat, per-unit classification ledger into named, describable
candidates: workflow candidates with ordered member stages, stage-context
attachments, permanent-instruction candidates, shared helper/context
candidates, obsolete-mechanism findings, and engine-pressure candidates —
per `references/synthesis-method.md`'s six buckets. This stage clusters and
names; it does not materialize files under the draft namespace — that is
`60-draft`'s job.

## What must become true here (durable outcome)

`output/candidates.md` exists, organized by the six buckets in
`references/synthesis-method.md`, with every classification record from
`../40-classify/output/classifications.ndjson` accounted for in exactly one
bucket appearance, traceable back to its `behavior_id`(s). No candidate is
named without at least one member record citing it.

## How to do it

Follow `references/synthesis-method.md`'s six buckets in order:

1. Group `workflow`-rung (and workflow-attached `stage`/`helper`) records
   into named workflow candidates — kebab-case name, a real trigger/outcome/
   completion description, ordered member stages.
2. Group `stage`-rung records within each workflow candidate; order them by
   the `trigger`→`outcome` chain across their source behavior units in
   `../30-normalize/output/behavior-units.normalized.ndjson`; note any
   genuinely ambiguous ordering call and why you made it.
3. Attach `stage-context`-rung records to the stage candidate their
   `workflow`+`stage` fields name. Flag any that name a stage with no
   corresponding stage candidate — do not invent one to hang it on.
4. List `agents-invariant`-rung records as permanent-instruction candidates.
   Do not draft them into any workflow package.
5. List `shared-helper`/`shared-context`-rung records with their contract
   (inputs, output shape, meaning) and consuming candidates; note whether a
   same-named `.sergeant/common/` entry already exists.
6. List `obsolete-mechanism`-disposition records with the mechanism, the
   settled fact replacing it, and where any surviving policy re-homes.
7. Carry `engine-gap`-rung records' `engine_gap` objects through unchanged
   as engine-pressure candidates; note (but do not force) any overlap in
   required capability between two or more of them.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
