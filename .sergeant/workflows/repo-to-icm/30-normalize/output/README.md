# Output — `30-normalize`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `behavior-units.normalized.ndjson` — the complete
rewritten corpus, one record per line, in the same field shape as
`../20-harvest/output/behavior-units.ndjson`, with every input unit
accounted for (rewritten, carried through, or split) per `../CONTEXT.md`
and `references/normalization-method.md`.

**Disposition:** `promote`

These are the workflow's declared "normalized behavior units" (§9.1's
`OUTPUT` list) — the input to classification and everything synthesized
after it. It must survive the merge; downstream consumers outside this
workflow's own run (review, measurement, a later re-classification) need
the normalized corpus, not just the raw harvest.
