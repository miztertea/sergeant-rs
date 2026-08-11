# Provenance -- sergeant-help-query

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

`sergeant-help-query` -- sourced directly from the `representation: workflow` record BU-0124 (its own `workflow` field is `null`, per convention; this is the record whose topic this candidate's name and description are drawn from).

## Stages

None -- no `representation: stage` record carries this candidate's `workflow` value. See `CONTEXT.md` "Zero materialized stages".

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `handle-failure-and-handoff` -- `BU-0128`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0125` -- a fixed five-way source-precedence order resolves disagreement among documentation sources
