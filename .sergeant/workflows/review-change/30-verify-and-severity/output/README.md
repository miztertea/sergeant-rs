# Output — `30-verify-and-severity`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `findings.md` — the finding set with each surviving
finding's staleness check and assigned severity (`blocker`/`major`/
`minor`) recorded.

**Disposition:** `promote`
