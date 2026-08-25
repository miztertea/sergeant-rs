# Output — `25-refute`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `findings.md` — the same typed finding set from
`20-panel`, updated in place with each row's final `status` and, where
`refuted`, the refuter's argument.

**Disposition:** `promote`
