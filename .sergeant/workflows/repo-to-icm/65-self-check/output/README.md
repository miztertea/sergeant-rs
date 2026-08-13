# Output — `65-self-check`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree apart from this file; a run of this stage
writes its artifact here in the materialized work surface — in this case,
written directly by the stage's own container rather than by an actor —
Git-tracked on the Work branch and reviewable in the diff like any other
change.

**Expected artifact:** `self-check-result.txt` — the container's captured
stdout+stderr from running `../scripts/validate-structure.py` (no path
argument) against this workflow's own tree: the validator's PASS/FAIL
result, and its findings if any.

**Disposition:** `evidence`

Per-run diagnostic evidence about this run's own worktree, not a
carry-through artifact another stage depends on structurally — `70-lint`
reads it for its own record (`output/lint-report.md`'s
"This workflow's own tree" heading), but nothing downstream fails to run
if this file is absent from a future merged history the way a `promote`d
manifest would.
