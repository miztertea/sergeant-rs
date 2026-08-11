# Troubleshoot Td Identity

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** The td executable resolved on PATH does not support the required flags.

**Outcome.** PATH is corrected to the required implementation rather than building a wrapper around the wrong one, until td create --help shows the required description/JSON/working-directory options.

**Completion.** Td create --help shows the required description/JSON/working-directory options.

## Zero materialized stages

No `representation: stage` record in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` carries this candidate's `workflow` value. Per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` and `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3, this is not resolved by inventing a stage: this package has no `NN-*/` directories and `workflow.toml` declares `stages = []`. See `provenance.md` for the evidence this candidate boundary rests on instead.

## Unattached stage-context evidence, not materialized

1 `stage-context` behavior_id(s), across 1 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## External shared dependencies (not part of this package)

- **5d td-capability-surface-check** (`BU-0132, BU-0209`) -- the fix this workflow's diagnosis ultimately restores compliance with. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.

## Note

There is no `representation: workflow` record for this candidate; its only member is the single unattached `stage-context` record BU-0173 -- an intentionally single-behavior workflow candidate, not reshaped into anything larger (`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` #16).
