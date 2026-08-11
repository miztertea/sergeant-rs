# Project Graphify

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** Sgt-graphify is invoked to extract and publish a project's knowledge graph.

**Outcome.** Publication is atomic-after-completion, never overlaps or destroys a source repo, and a failed or incomplete run is never promoted to the published location.

**Completion.** Publish-graph stops the run before publication if extraction produced zero matched repos, or any repo's extraction failed.

## How its stages relate

Ordered, trigger-to-outcome:

1. **publish-graph** (`01-publish-graph/`) -- Trigger: extraction produces zero matched repos, or any repo's extraction fails. Outcome: the run stops before publication rather than silently merging and publishing an incomplete graph.

## Unattached stage-context evidence, not materialized

3 `stage-context` behavior_id(s), across 3 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Workflow-local helper machinery (not separately packaged)

12 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.
