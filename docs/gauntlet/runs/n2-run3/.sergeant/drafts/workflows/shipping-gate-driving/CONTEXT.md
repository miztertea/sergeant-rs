# Shipping Gate Driving

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** The coordinator drives a no-mistakes shipping gate to completion for dispatched (or direct-mode) work.

**Outcome.** The gate is started at most once per precondition-satisfied run, polled rather than re-issued, and findings are routed by disposition.

**Completion.** Group-remediation converges remediation to one worker per shared root cause, rechecked before merge, escalating to a human after two unsuccessful cycles.

## How its stages relate

Ordered, trigger-to-outcome:

1. **group-remediation** (`01-group-remediation/`) -- Trigger: multiple findings share the same root cause. Outcome: remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely.

## Unattached stage-context evidence, not materialized

5 `stage-context` behavior_id(s), across 3 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Note

Notably the actual run-start (`start-run`) and gate-driving (`drive-gate`) mechanics that most of this workflow's outcome description is drawn from were classified `stage-context`, not `stage` -- only the remediation-convergence checkpoint reached the `stage` rung. See `provenance.md`.
