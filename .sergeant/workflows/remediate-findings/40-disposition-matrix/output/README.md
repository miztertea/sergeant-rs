# Output — `40-disposition-matrix`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `disposition-matrix.md` — every ingested id, its
disposition, its reason, and (where accepted) its fix commit — the
completeness proof.

**Disposition:** `promote`
