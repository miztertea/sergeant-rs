# Output — `00-require-terminal`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — every targeted repo verified safely terminal and ownership-clean, handshake acknowledgement verified and sealed, each repo's surface removed, and (only once every repo is done) whole-task fleet state retired. This is the sole stage in the workflow (N1 adjudication A4 folded the five machinery stages that previously produced their own `evidence`-disposition artifacts — ownership verification, handshake verification, surface removal, state retirement, plus the two stages moved in from `monitor-fleet` under A7 — into this stage's own record as helper-invocation output).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"). It inherits `promote` from the former `40-retire-state` stage, whose retirement-of-whole-task-state was this workflow's only deliverable-grade artifact; the folded evidence steps (ownership/handshake verification, surface removal) remain part of this same record as intermediate decisions, not separately promoted.
