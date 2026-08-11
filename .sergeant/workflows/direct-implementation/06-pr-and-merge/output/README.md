# Output — `06-pr-and-merge`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — A PR is opened and merged per repository convention, and handoff/PR/merge/deployment/cleanup outcomes are recorded against the owning tracked task (folds the demoted `07-record-outcomes` checkpoint, N1 adjudication A4).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly). This stage is now the workflow's last stage (`07-record-outcomes` demoted and folded in), so its output carries the `promote` disposition `07-record-outcomes`'s output previously carried.
