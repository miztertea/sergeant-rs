# Output — `40-apply-and-acknowledge`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — target validation, response publication, and delivery/acceptance (helper invocations, N1 adjudication A4), the decision applied once with truthful status restored and applied id/generation/status recorded and acknowledged, then archiving, coordinator notification, and relaunch-if-needed (further helper invocations).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"). It inherits `promote` from the former `70-relaunch-if-needed` stage, whose finalizer-convergence-or-refusal was this workflow's only deliverable-grade artifact; the folded evidence steps (validate-target, publish-response, deliver-and-accept, apply-and-acknowledge itself, archive-evidence, notify-coordinator) remain part of this same record as intermediate decisions, not separately promoted.
