# Output — `30-fix-confirmed`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `fixes.md` — every confirmed finding's id linked to
its fix commit(s), or recorded unfixed with a reason; the re-run
validation's real output; any recommended follow-up intents the fixer
noticed but did not act on.

**Disposition:** `promote`
