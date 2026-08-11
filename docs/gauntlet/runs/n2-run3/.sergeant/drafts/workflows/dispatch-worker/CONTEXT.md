# Dispatch Worker

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** Dispatch mode has been selected for a task and work has been decomposed by owning repository.

**Outcome.** Each owning repository ends up with a durably launched, evidence-backed worker running under one stable canonical intent, or dispatch fails closed before mutating anything.

**Completion.** Every target repo either has a spawned worker with recorded launch evidence and a generation-tracked gate identity, or the dispatch aborted with no partial state left behind.

## How its stages relate

Ordered, trigger-to-outcome:

1. **intent-file-gate** (`01-intent-file-gate/`) -- Gates any mutating dispatch action behind a validated intent file when the objective touches a sensitive category -- the earliest possible checkpoint, since it must run before task creation or worker spawn.
2. **create-td-tasks** (`02-create-td-tasks/`) -- All-or-nothing td task creation across selected repos, with rollback on partial failure, explicitly before any worker is spawned.
3. **record-canonical-intent** (`03-record-canonical-intent/`) -- A dispatch's canonical intent is recorded at dispatch-creation time (BU-0135's trigger: 'a dispatch is created'), after the two preconditions above and before spawn; that same intent then stays stable and governs every later dispatched action -- implementation, review, PR, successor, recovery, shipping-gate (BU-0040, BU-0303).
4. **spawn-worker** (`04-spawn-worker/`) -- Trigger: work has been decomposed by repository. Outcome: one dispatched worker launched per repo (BU-0007); the four converging spawn-failure paths are handled without partial state (BU-0295).
5. **escalate-undecided-seam** (`05-escalate-undecided-seam/`) -- Triggered while a worker is already running and needs to establish an undecided public behavioral seam -- necessarily after spawn.
6. **report-terminal-status** (`06-report-terminal-status/`) -- Triggered when a worker reaches a terminal outcome -- the last checkpoint in the sequence.

## Unattached stage-context evidence, not materialized

14 `stage-context` behavior_id(s), across 10 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Workflow-local helper machinery (not separately packaged)

22 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.

## External shared dependencies (not part of this package)

- **5a toolbelt-command-resolution** (`BU-0020, BU-0056`) -- invokes sgt-* commands. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.
- **5e shared-review-axis-definition** (`BU-0095, BU-0300`) -- brief generation. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.
