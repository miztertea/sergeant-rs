# Provenance -- dispatch-worker

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `dispatch-worker`. The workflow boundary is inferred from 47 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"dispatch-worker"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-intent-file-gate

`BU-0140` (`representation: stage`). Gates any mutating dispatch action behind a validated intent file when the objective touches a sensitive category -- the earliest possible checkpoint, since it must run before task creation or worker spawn.

### 02-create-td-tasks

`BU-0284` (`representation: stage`). All-or-nothing td task creation across selected repos, with rollback on partial failure, explicitly before any worker is spawned.

### 03-record-canonical-intent

`BU-0040, BU-0135, BU-0303` (`representation: stage`). A dispatch's canonical intent is recorded at dispatch-creation time (BU-0135's trigger: 'a dispatch is created'), after the two preconditions above and before spawn; that same intent then stays stable and governs every later dispatched action -- implementation, review, PR, successor, recovery, shipping-gate (BU-0040, BU-0303).

### 04-spawn-worker

`BU-0007, BU-0295` (`representation: stage`). Trigger: work has been decomposed by repository. Outcome: one dispatched worker launched per repo (BU-0007); the four converging spawn-failure paths are handled without partial state (BU-0295).

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0071` -- launch evidence never overclaims model readiness
- `BU-0072` -- launch evidence never overclaims variant verification

### 05-escalate-undecided-seam

`BU-0281` (`representation: stage`). Triggered while a worker is already running and needs to establish an undecided public behavioral seam -- necessarily after spawn.

### 06-report-terminal-status

`BU-0283` (`representation: stage`). Triggered when a worker reaches a terminal outcome -- the last checkpoint in the sequence.

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `plan-and-decompose` -- `BU-0006`
- `monitor-and-reconcile` -- `BU-0008`
- `validate-harness-selection` -- `BU-0057`
- `validate-model-compatibility` -- `BU-0059`
- `bind-coordinator-pane` -- `BU-0060, BU-0075, BU-0076, BU-0077`
- `record-launch-evidence` -- `BU-0070`
- `acquire-drain-lock` -- `BU-0105, BU-0289`
- `route-before-implementation` -- `BU-0280`
- `handle-notification` -- `BU-0305`
- `handle-crash-exit` -- `BU-0306`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0058` -- model/variant resolution & validation
- `BU-0065` -- model/variant resolution & validation
- `BU-0066` -- model/variant resolution & validation
- `BU-0068` -- model/variant resolution & validation
- `BU-0069` -- model/variant resolution & validation
- `BU-0197` -- model/variant resolution & validation
- `BU-0273` -- td task creation mechanics
- `BU-0285` -- td task creation mechanics
- `BU-0290` -- td task creation mechanics
- `BU-0297` -- td task creation mechanics
- `BU-0298` -- td task creation mechanics
- `BU-0278` -- worktree/treehouse setup
- `BU-0291` -- worktree/treehouse setup
- `BU-0292` -- worktree/treehouse setup
- `BU-0293` -- worktree/treehouse setup
- `BU-0299` -- worktree/treehouse setup
- `BU-0287` -- coordinator-pane / dispatch-invocation plumbing
- `BU-0288` -- coordinator-pane / dispatch-invocation plumbing
- `BU-0296` -- coordinator-pane / dispatch-invocation plumbing
- `BU-0279` -- cross-repo dependency & credential handling
- `BU-0294` -- cross-repo dependency & credential handling
- `BU-0307` -- gate identity publishing

## Shared-helper/shared-context evidence, external to this package

- **5a toolbelt-command-resolution** (`BU-0020, BU-0056`) -- invokes sgt-* commands.
- **5e shared-review-axis-definition** (`BU-0095, BU-0300`) -- brief generation.
