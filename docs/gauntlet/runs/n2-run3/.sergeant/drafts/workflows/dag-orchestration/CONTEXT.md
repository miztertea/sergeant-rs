# Dag Orchestration

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A DAG stage declares an after: dependency.

**Outcome.** The stage becomes ready to dispatch only once its named predecessor stages have completed.

**Completion.** Stage-dependency-gate, advanced automatically by sgt-watch.

## How its stages relate

Ordered, trigger-to-outcome:

1. **stage-dependency-gate** (`01-stage-dependency-gate/`) -- Trigger: a DAG stage declares an after: dependency. Outcome: the stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by sgt-watch.

## Workflow-local helper machinery (not separately packaged)

2 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.
