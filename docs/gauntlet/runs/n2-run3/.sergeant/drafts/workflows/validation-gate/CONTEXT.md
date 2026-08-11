# Validation Gate

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** Dispatched (or direct-mode) work reaches readiness for shipping validation.

**Outcome.** Exactly one validation launch runs to completion under a coordinator-verified, auditable transport, and readiness is durably published only once every gate has genuinely passed.

**Completion.** Readiness evidence anchored to a real, committed HEAD is recorded before the coordinator is notified.

## How its stages relate

Ordered, trigger-to-outcome:

1. **launch-validation** (`01-launch-validation/`) -- Trigger: a dispatched worker reaches readiness / sgt-validate is invoked at readiness. Outcome: a validation-only boundary runs in a coordinator-owned, split pane, never auto-approved, with redundant stages skipped by a defined default set.
2. **publish-readiness** (`02-publish-readiness/`) -- Trigger: native validation and independent reviews all pass. Outcome: readiness is durably recorded with intent/head/review evidence before the coordinator is notified.

## Unattached stage-context evidence, not materialized

2 `stage-context` behavior_id(s), across 2 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Workflow-local helper machinery (not separately packaged)

1 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.
