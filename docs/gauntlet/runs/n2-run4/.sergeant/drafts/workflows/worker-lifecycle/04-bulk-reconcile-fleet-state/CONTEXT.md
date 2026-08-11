# 04-bulk-reconcile-fleet-state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the interactive fleet-watch loop --sync-all runs

**Outcome:** panes are stopped only after identity verification, and ambiguous interrupted records wait out a grace period before being marked failed

**Statement (the operative rule):** Bulk reconciliation (the interactive fleet-watch loop) syncs worktree status into fleet state, stops only identity-verified `done` or `failed` worker panes, and marks interrupted `dispatched` records failed only when they have neither a worktree nor an owned live pane after a default 300-second grace period.

## What must become true here (durable outcome)

Panes are stopped only after identity verification, and ambiguous interrupted records wait out a grace period before being marked failed — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0142`: Bulk reconciliation preserves `needs_input`, `blocked`, and `orphaned` worktrees, and dispatch runs this reconciliation automatically before creating new tasks.
- `BU-0603`: The interactive fleet-watch loop --sync-all reconciles every task directory under the fleet root and reports how many it processed; unlike the interactive watch loop, it does not fail the invocation based on any individual repo's terminal status.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0607`: The interactive fleet-watch loop --sync <task-id> requires the named task directory to exist, dying with a usage-style error naming the fleet directory it looked in if the task cannot be found, rather than silently doing nothing.

