# Provenance -- validation-gate

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `validation-gate`. The workflow boundary is inferred from 13 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"validation-gate"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-launch-validation

`BU-0042, BU-0161` (`representation: stage`). Trigger: a dispatched worker reaches readiness / sgt-validate is invoked at readiness. Outcome: a validation-only boundary runs in a coordinator-owned, split pane, never auto-approved, with redundant stages skipped by a defined default set.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0162` -- exactly one launch per task/repo pair, concurrent attempts fail closed
- `BU-0163` -- default transport never exposes intent via argv
- `BU-0164` -- missing --intent-file support fails closed with full diagnostic, no partial state
- `BU-0165` -- an argv-exposure consent applies to exactly one invocation, cannot silently persist
- `BU-0166` -- transport choice is durably auditable and the executing build is re-verified against it
- `BU-0169` -- rollback on pre-commit failure is scoped strictly to provably-owned artifacts

### 02-publish-readiness

`BU-0160` (`representation: stage`). Trigger: native validation and independent reviews all pass. Outcome: readiness is durably recorded with intent/head/review evidence before the coordinator is notified.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0309` -- readiness evidence anchored to a committed HEAD, never a working-tree diff

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `post-readiness-remediation` -- `BU-0043`
- `claim-validation-ownership` -- `BU-0167`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0168` -- every ownership claim/release is durably logged; a release token is single-use
