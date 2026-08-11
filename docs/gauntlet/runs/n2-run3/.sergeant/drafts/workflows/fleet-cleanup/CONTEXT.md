# Fleet Cleanup

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** Sgt-cleanup is invoked for a task.

**Outcome.** Cleanup proceeds only once every named precondition holds -- terminal proof, staged evidence, a converged or explicitly retired response handshake, and (when applicable) callback completion -- never as a shortcut for a nonterminal worker state.

**Completion.** Cleanup-preconditions.

## How its stages relate

Ordered, trigger-to-outcome:

1. **cleanup-preconditions** (`01-cleanup-preconditions/`) -- Trigger: sgt-cleanup is invoked for a task. Outcome: cleanup proceeds only once every named precondition holds, and never as a shortcut for a nonterminal worker state.

## Unattached stage-context evidence, not materialized

9 `stage-context` behavior_id(s), across 5 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Note

Worth naming plainly: the response-handshake, callback-gate, seal, and sealed-failure-recovery mechanics that most of this workflow's outcome description is drawn from all classified as `stage-context`, not `stage`. See `provenance.md`.
