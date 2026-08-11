# Provenance -- dag-orchestration

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `dag-orchestration`. The workflow boundary is inferred from 3 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"dag-orchestration"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-stage-dependency-gate

`BU-0203` (`representation: stage`). Trigger: a DAG stage declares an after: dependency. Outcome: the stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by sgt-watch.

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0201` -- a DAG name must not collide with another project's DAG name
- `BU-0202` -- a stage's brief source is one of two alternatives, resolved by whether td is set
