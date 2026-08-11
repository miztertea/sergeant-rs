# Fleet Monitor And Reconcile

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** Sgt-watch runs to snapshot fleet state (--snapshot), reconcile it in bulk (--sync-all), or assess a worker's health.

**Outcome.** Busy/health/notification state is reported strictly from verified evidence -- never fabricated as idle, healthy, or a known basis value when the verification conditions don't all hold.

**Completion.** No stage in this corpus names it -- see provenance.md.

## Zero materialized stages

No `representation: stage` record in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` carries this candidate's `workflow` value. Per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` and `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3, this is not resolved by inventing a stage: this package has no `NN-*/` directories and `workflow.toml` declares `stages = []`. See `provenance.md` for the evidence this candidate boundary rests on instead.

## Unattached stage-context evidence, not materialized

2 `stage-context` behavior_id(s), across 2 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Workflow-local helper machinery (not separately packaged)

5 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.
