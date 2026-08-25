# Output — `40-fix-with-regression-test`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `fix.md` — the fix commit(s), the regression test
and its seam (or the recorded seam-absence finding), and the re-run
feedback loop's result.

**Disposition:** `promote`
