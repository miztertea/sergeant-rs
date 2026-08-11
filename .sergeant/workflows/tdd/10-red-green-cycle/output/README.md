# Output — `10-red-green-cycle`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — One seam, one test, one minimal implementation, vertical slices only.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly).

**Curation note** (`docs/icm/promotion-spec-2026-08-11.md` §1): this is `tdd`'s true closing stage and it declares a `promote` output, but the workflow has no dedicated finalize step (D9, an open question, not a Rule). Disposition here is applied by human review at merge time, not mechanically.
