# Output — `10-evaluate`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — the condition validated against its field/value allowlist, one of six typed condition kinds evaluated (helper invocations, N1 adjudication A4), the outcome classified met/unmet/escalate/failed, and the worker resumed on a met outcome.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"). It inherits `promote` from the former `30-resume` stage, whose worker-resumption was this workflow's only deliverable-grade artifact; the folded evidence steps (validate-condition, evaluate itself, classify-outcome) remain part of this same record as intermediate decisions, not separately promoted.
