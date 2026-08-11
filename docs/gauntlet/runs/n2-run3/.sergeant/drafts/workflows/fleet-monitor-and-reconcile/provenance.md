# Provenance -- fleet-monitor-and-reconcile

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `fleet-monitor-and-reconcile`. The workflow boundary is inferred from 7 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"fleet-monitor-and-reconcile"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

None -- no `representation: stage` record carries this candidate's `workflow` value. See `CONTEXT.md` "Zero materialized stages".

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `sync-all` -- `BU-0141`
- `assess-worker-health` -- `BU-0144`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0062` -- busy:true only when all three verification conditions hold
- `BU-0063` -- unverified conditions report null/unknown, never a fabricated idle
- `BU-0064` -- an unrecognized observed condition falls back to the null basis
- `BU-0143` -- concurrent updates degrade notification to a delayed wakeup, never a duplicate
- `BU-0145` -- stale progress evidence is diagnosed as stalled, not reclassified to terminal
