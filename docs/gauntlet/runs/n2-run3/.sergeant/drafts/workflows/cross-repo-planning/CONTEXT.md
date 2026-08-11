# Cross Repo Planning

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A requested outcome is being decomposed across repositories.

**Outcome.** Exactly one repository is named as owning each required behavior, and a repository is included only when it must actually change or produce delivery evidence.

**Completion.** Ownership assignment for every required behavior.

## Zero materialized stages

No `representation: stage` record in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` carries this candidate's `workflow` value. Per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` and `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3, this is not resolved by inventing a stage: this package has no `NN-*/` directories and `workflow.toml` declares `stages = []`. See `provenance.md` for the evidence this candidate boundary rests on instead.

## Unattached stage-context evidence, not materialized

1 `stage-context` behavior_id(s), across 1 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Note

There is no `representation: workflow` record for this candidate either; its only member is the single unattached `stage-context` record BU-0267 (`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` #17).
