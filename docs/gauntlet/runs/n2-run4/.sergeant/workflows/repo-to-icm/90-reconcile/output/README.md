# Output — `90-reconcile`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifacts here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change. This is the workflow's last
stage: after these artifacts and `../scripts/finalize.py` are written, the
run is closed.

**Expected artifacts:**

| File | Disposition |
|---|---|
| adjudication-log.md | promote |
| measurement-package.md | promote |
| grammar-pressure.ndjson | promote |

`adjudication-log.md` — every `80-adversarial-review` finding's disposition
(accept/reject/park) and reason. `measurement-package.md` — the
internally-computable `§9.9` dimensions (explicitly naming the five that
require the separate, later blind comparison against `reference-corpus/`
and are not covered here), ending with `../scripts/finalize.py`'s recorded
output. `grammar-pressure.ndjson` — every surviving engine-gap claim,
behavior-level and meta-level, as full six-field records
(`docs/gauntlet/contracts/N2.md` Outcome §4; exact wrapper shape in
`references/reconciliation-method.md` §3).

All three are `promote`: they are this run's final deliverable record — the
measurement package and grammar-pressure report this milestone exists to
produce, and the adjudication log that makes every accepted repair
traceable to a reason. None of them is scratch.
