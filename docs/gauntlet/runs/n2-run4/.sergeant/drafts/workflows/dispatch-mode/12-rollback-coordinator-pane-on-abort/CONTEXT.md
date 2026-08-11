# 12-rollback-coordinator-pane-on-abort

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** dispatch creates a new managed coordinator window and a later step aborts

**Outcome:** cleanup is scoped precisely to what this invocation created, and covers every later abort point from the moment the pane is bound

**Statement (the operative rule):** The managed-coordinator-pane rollback trap removes only the exact window this invocation created, never a window Sergeant merely selected/adopted, and is installed as an EXIT trap immediately after pane binding since several later preflight steps can still abort.

## What must become true here (durable outcome)

Cleanup is scoped precisely to what this invocation created, and covers every later abort point from the moment the pane is bound — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0296`: Once every target repo has been successfully dispatched, the managed-coordinator-pane rollback is disarmed, since the pane is now owned by live fleet state rather than by this invocation's own error-cleanup scope.

