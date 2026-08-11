# Provenance -- project-graphify

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `project-graphify`. The workflow boundary is inferred from 16 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"project-graphify"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-publish-graph

`BU-0250` (`representation: stage`). Trigger: extraction produces zero matched repos, or any repo's extraction fails. Outcome: the run stops before publication rather than silently merging and publishing an incomplete graph.

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `diagnose-output-path` -- `BU-0184`
- `publish-output` -- `BU-0198`
- `confirm-output-path` -- `BU-0262`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0133` -- success requires both named output artifacts to exist
- `BU-0196` -- an unsafe repo name is rejected before use as a path prefix
- `BU-0245` -- readers never see a torn/partially-written output
- `BU-0246` -- an invalid repo name fails that repo's extraction with a clear error
- `BU-0247` -- output colliding with a source repo path fails closed
- `BU-0248` -- extraction runs against an exclusion-applied copy, never the live tree
- `BU-0249` -- missing LLM API key degrades to code-only indexing, doesn't abort
- `BU-0251` -- an incomplete staged output is never promoted
- `BU-0252` -- existing wiki/memory/ subdirectories survive publish
- `BU-0253` -- a mid-swap failure leaves the previous output intact
- `BU-0254` -- the symlink swap is atomic, old target removed only after the new one is confirmed live
- `BU-0263` -- success confirmed by artifact presence, not exit code
