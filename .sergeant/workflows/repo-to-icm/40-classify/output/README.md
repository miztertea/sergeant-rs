# Output — `40-classify`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `classifications.ndjson` — one classification record
per line, per `references/classification-record-shape.md`, covering every
unit in `../30-normalize/output/behavior-units.normalized.ndjson` exactly
once.

**Disposition:** `promote`

The classification ledger is this workflow's own declared "classification
ledger" output (§9.1's `OUTPUT` list) and the input every downstream stage
(synthesis, drafting, review, reconciliation) traces back to by
`behavior_id`. It must survive the merge for that traceability to mean
anything after the run is over.
