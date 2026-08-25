# Output — `35-re-verify`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `re-verify.md` — the re-attack's findings (if any),
the test-honesty audit's results per new/changed test, and — when clean —
an explicit positive record of what was attacked and found not to be
wrong.

**Disposition:** `promote`
