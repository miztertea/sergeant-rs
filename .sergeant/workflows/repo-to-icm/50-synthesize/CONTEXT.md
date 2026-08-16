# 50-synthesize: cluster classified units into candidates

## Inputs

| File | Layer | Why |
|---|---|---|
| references/synthesis-method.md | L3 | the seven buckets and how to cluster/order/name candidates within each |
| ../_config/icm-ladder.md | L3 | bucket 6's obsolete-mechanism naming requirement and bucket 7's engine-gap template both refer back to this |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| ../40-classify/output/classifications.ndjson | L4 | upstream artifact produced by `40-classify` — the per-unit classification records this stage clusters |
| ../30-normalize/output/behavior-units.normalized.ndjson | L4 | upstream artifact produced by `30-normalize` — the normalized `statement`/`trigger`/`outcome` text needed to name candidates and order stages (classification records alone carry no readable behavior text) |

## Purpose

Turn the flat, per-unit classification ledger into named, describable
candidates: workflow candidates with ordered member stages, stage-context
attachments, permanent-instruction candidates, shared helper/context
candidates, obsolete-mechanism findings, and engine-pressure candidates —
per `references/synthesis-method.md`'s seven buckets. This stage clusters and
names; it does not materialize files under the draft namespace — that is
`60-draft`'s job.

## Bounded judgment

Apply `@@bounded-judgment`.

A governing constraint (J5, `../_config/icm-ladder.md` §6.6 / `docs/icm/
record-shapes.md` §6 rule 4): clustering groups by behavioral contract —
what a unit does, for whom — never by originating source file. A cluster
that reproduces the inventory's own file list one-for-one is a defect this
stage must name, not something it may quietly reshape into a tidier bucket.

### J2 — delegated to this stage
- Ordering stage candidates within a workflow candidate by the
  `trigger`→`outcome` chain across member behavior units, recording a
  one-line reason for any genuinely ambiguous ordering call.
- Naming each candidate (kebab-case, checked for collision against every
  other candidate this run mints and against both admitted and draft
  workflow trees).
- Recording an `## Unattached records` entry when a `stage-context`,
  `helper`, or `stage` record names a `workflow`/`stage` with no
  corresponding candidate, rather than inventing one to attach it to.
- Flagging the over-promotion tell on bucket 5 (shared helper/context)
  groupings, naming which files mirror one-to-one.

### J1 — local choices allowed
- The order buckets are worked in beyond "in order" (the seven-bucket
  sequence itself is fixed by `references/synthesis-method.md`), and
  internal formatting of `output/candidates.md`.

### J0 — must become `needs_input`
- `../40-classify/output/classifications.ndjson` opens with
  `# AMBIGUOUS — NOT RESOLVED` — do not proceed; follow `../_config/
  run-discipline.md` §2.

### Completion boundary
This stage may complete only when every classification record from
`../40-classify/output/classifications.ndjson` appears in exactly one
bucket appearance (including an `## Unattached records` appearance where
applicable), and no candidate is named without at least one member record
citing it.

### Decision evidence
`output/candidates.md` — its per-bucket rationale for ordering, naming, and
any `## Unattached records` entries — is this stage's decision record.

## What must become true here (durable outcome)

`output/candidates.md` exists, organized by the seven buckets in
`references/synthesis-method.md`, with every classification record from
`../40-classify/output/classifications.ndjson` accounted for in exactly one
bucket appearance, traceable back to its `behavior_id`(s). No candidate is
named without at least one member record citing it.

## How to do it

0. If `../40-classify/output/classifications.ndjson` opens with `#
   AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

Follow `references/synthesis-method.md`'s seven buckets in order:

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
