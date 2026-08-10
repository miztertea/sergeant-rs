# Output — `20-harvest`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `behavior-units.ndjson` — one behavior-unit record
per line, in the shape `../_config/evidence-policy.md` defines, covering
every `decompose`-dispositioned file/partition from `10-inventory` (or
recording, for any not reached, why not — see `../CONTEXT.md`).

**Disposition:** `promote`

Extracted behavior units are this workflow's central deliverable — the
"normalized behavior units" and everything downstream (classification,
synthesis, draft workflows) traces back to this file's records by `id`. It
must survive the merge for that traceability to mean anything after the
run is over.
