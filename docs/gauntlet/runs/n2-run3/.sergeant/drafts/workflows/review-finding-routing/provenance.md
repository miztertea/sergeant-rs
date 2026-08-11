# Provenance -- review-finding-routing

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `review-finding-routing`. The workflow boundary is inferred from 16 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"review-finding-routing"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-route-finding

`BU-0096` (`representation: stage`). Trigger: a dispatched worker produces a review finding artifact. Outcome: actionable findings become owning-repo td tasks with durably published blocking guidance.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0103` -- a failed route retains parsed/sanitized findings with an exact retry command
- `BU-0312` -- a malformed/failed-routing review artifact escalates rather than being silently logged

### 02-reconcile-hand-edit

`BU-0101` (`representation: stage`). Trigger: a stored finding card has been modified outside the router since it last wrote it. Outcome: the human-edited content is preserved (not overwritten) and flagged for human reconciliation.

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0090` -- disposition maps deterministically to one of four outcomes
- `BU-0091` -- repeated-ID findings update, not duplicate, the existing card
- `BU-0092` -- manual/repo labels survive a rerun
- `BU-0093` -- a hidden-state card is resurfaced before its body refreshes
- `BU-0097` -- non-actionable findings produce no card; malformed bodies rejected; credential-shaped content redacted
- `BU-0098` -- severity normalized to three canonical levels
- `BU-0099` -- dedup key dimensioned enough to avoid generic-ID collisions
- `BU-0100` -- digest detects a hand-modified stored card
- `BU-0102` -- a match against a closed card reopens it, surfaced, not silently recreated
- `BU-0104` -- a retried retained artifact re-validates/re-digests before td is touched
- `BU-0301` -- an axis without guidance text fails loudly
- `BU-0302` -- a high-severity finding always blocks

## Shared-helper/shared-context evidence, external to this package

- **5e shared-review-axis-definition** (`BU-0095, BU-0300`) -- finding acceptance/routing.
