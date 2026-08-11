# Output — `40-validate`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. Empty at
draft time — this describes artifact shape for **this candidate's own
future runs** once promoted, not an artifact of the `repo-to-icm` run that
materialized this draft package.

**Expected artifact:** a validation record — the outcome of the single
dedicated validation run, and, if a post-readiness HEAD change triggered
rereview, a record of that rereview and its outcome.

**Disposition:** `promote`

`50-reconcile-and-deliver`'s own precondition ("verified terminal state")
depends on this record existing and being settled.
