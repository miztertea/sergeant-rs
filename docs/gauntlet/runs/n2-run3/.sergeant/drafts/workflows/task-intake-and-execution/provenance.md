# Provenance -- task-intake-and-execution

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `task-intake-and-execution`. The workflow boundary is inferred from 14 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"task-intake-and-execution"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-resolve-context

`BU-0025` (`representation: stage`). Trigger: a task is brought to the session. Outcome: context is fully loaded before an execution mode is chosen.

### 02-resolve-task

`BU-0026` (`representation: stage`). Trigger: context has been loaded. Outcome: an existing canonical td task is reused, or a new one created only otherwise.

### 03-reconcile-before-start

`BU-0028` (`representation: stage`). Trigger: an execution mode has been chosen, before starting work. Outcome: existing state is reconciled and reused rather than duplicated.

### 04-execute

`BU-0031` (`representation: stage`). Trigger: decisions have been confirmed. Outcome: execution proceeds via the mode-appropriate path.

### 05-direct-mode-implementation

`BU-0012` (`representation: stage`). Trigger: direct mode is active. Outcome: the owning td task is claimed/created and implementation proceeds test-first.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0010` -- context/td state loaded before any edit
- `BU-0011` -- in-progress work by other workers/worktrees reconciled, not duplicated/raced

### 06-handle-decision-gate

`BU-0034` (`representation: stage`). Trigger: a worker reaches needs_input, blocked, or an ask-user gate. Outcome: only genuinely missing decisions are solicited and recorded in td; remediation continues without redundant re-asks.

### 07-direct-mode-delivery

`BU-0015` (`representation: stage`). Trigger: a direct-mode implementation is ready for delivery. Outcome: delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0014` -- direct-mode work passes the same validation/review/gate steps as dispatched work
- `BU-0016` -- handoff/PR/merge/deployment/cleanup outcomes durably recorded

### 08-reconcile-and-deliver

`BU-0035` (`representation: stage`). Trigger: work has reached a terminal or deliverable state. Outcome: cleanup runs only after terminal state and evidence preservation are verified.

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `invoke-toolbelt-command` -- `BU-0021`
- `monitor-progress` -- `BU-0033`

## Shared-helper/shared-context evidence, external to this package

- **5a toolbelt-command-resolution** (`BU-0020, BU-0056`) -- invokes sgt-* commands.
