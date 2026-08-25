# Output — `60-re-verify-and-postmortem`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `re-verify-and-postmortem.md` — any confirmed
finding's fix commit (or recorded-unfixed reason), the re-attack and
test-honesty audit results, the closing checklist, the root-cause
postmortem, and any architectural recommendation.

**Disposition:** `promote`
