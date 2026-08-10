# Output — `40-escalate-or-continue`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — a new gate published only when a monotonic generation actually advanced, the handshake acknowledged/accepted/acted-on-once/marked-complete; or, on the concluding path, handoff evidence recorded from the verified work surface with bounded readiness (helper invocation, N1 adjudication A4).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"). It inherits `promote` from the former `50-publish-result` stage, whose handoff-evidence recording was this workflow's only deliverable-grade artifact on the concluding path; on the escalating path this same record documents the published gate as Work-branch evidence.
