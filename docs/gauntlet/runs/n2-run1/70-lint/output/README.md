# Output — `70-lint`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `lint-report.md` — per candidate package: the
validator's initial findings, each classified mechanical-fixed or
substantive-remaining per `references/mechanical-vs-substantive.md`, and
the final validator result after mechanical repairs.

**Disposition:** `promote`

This is the raw evidence for the "Draft validity" measurement dimension
(§9.9) and a direct input to `80-adversarial-review` (so it does not
re-litigate already-fixed mechanical defects) and `90-reconcile`'s
measurement package. It must survive the merge for either to be checkable
after the run.
