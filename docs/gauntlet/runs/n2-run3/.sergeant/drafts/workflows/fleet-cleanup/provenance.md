# Provenance -- fleet-cleanup

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `fleet-cleanup`. The workflow boundary is inferred from 10 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"fleet-cleanup"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-cleanup-preconditions

`BU-0171` (`representation: stage`). Trigger: sgt-cleanup is invoked for a task. Outcome: cleanup proceeds only once every named precondition holds, and never as a shortcut for a nonterminal worker state.

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `complete-response-handshake` -- `BU-0186`
- `retire-response-handshake` -- `BU-0187, BU-0188, BU-0189, BU-0190`
- `callback-completion-gate` -- `BU-0230, BU-0231`
- `seal-before-delete` -- `BU-0232`
- `recover-from-sealed-failure` -- `BU-0233`
