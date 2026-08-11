# Provenance -- shipping-gate-driving

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `shipping-gate-driving`. The workflow boundary is inferred from 6 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"shipping-gate-driving"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-group-remediation

`BU-0282` (`representation: stage`). Trigger: multiple findings share the same root cause. Outcome: remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely.

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `start-run` -- `BU-0080, BU-0082`
- `drive-gate` -- `BU-0083, BU-0084`
- `recover-from-failure` -- `BU-0087`
