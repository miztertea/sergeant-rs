# Provenance -- troubleshoot-td-identity

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `troubleshoot-td-identity`. The workflow boundary is inferred from 1 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"troubleshoot-td-identity"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

None -- no `representation: stage` record carries this candidate's `workflow` value. See `CONTEXT.md` "Zero materialized stages".

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `diagnose-wrong-td` -- `BU-0173`

## Shared-helper/shared-context evidence, external to this package

- **5d td-capability-surface-check** (`BU-0132, BU-0209`) -- the fix this workflow's diagnosis ultimately restores compliance with.
