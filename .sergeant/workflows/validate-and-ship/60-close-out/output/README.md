# Output — `60-close-out`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — Stop driving at `checks-passed`; on `failed`/`cancelled`, fix on the same branch and re-drive; summarize what the pipeline found and fixed; any coordinator ownership transfer during the run durably logged (folds the demoted `90-handover-log` checkpoint, N1 adjudication A4).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`.sergeant/common/contexts/icm-policy.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly). This stage is now the workflow's last stage (`90-handover-log` demoted and folded in), so its output carries the `promote` disposition `90-handover-log`'s output previously carried.
