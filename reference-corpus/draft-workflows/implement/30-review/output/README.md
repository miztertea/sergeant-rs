# Output — `30-review`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — The change is reviewed, and the verified change is committed to the current branch (folds the demoted `40-commit` checkpoint, N1 adjudication A4).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly). This stage is now the workflow's last stage (`40-commit` demoted and folded in), so its output carries the `promote` disposition `40-commit`'s output previously carried.
