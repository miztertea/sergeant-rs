# Provenance -- skill-adoption

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

`skill-adoption` -- sourced directly from the `representation: workflow` record BU-0119 (its own `workflow` field is `null`, per convention; this is the record whose topic this candidate's name and description are drawn from).

## Stages

None -- no `representation: stage` record carries this candidate's `workflow` value. See `CONTEXT.md` "Zero materialized stages".

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0121` -- each worker harness discovers the same canonical .agents/skills/ tree through its own harness-appropriate path; no install step ever writes to global user config
