# 15-handle-spawn-failure

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** any of the four named worker-spawn failure modes occurs

**Outcome:** every spawn failure path converges on the same explicit orphaning + evidence-recording sequence, never a silent or ambiguous half-started worker

**Statement (the operative rule):** If any step of spawning a worker fails — no pane returned, a notification-target creation race, failure to capture exact pane identity, or the worker not acknowledging its durable notification in time — dispatch stops any Claude background session the worker may have started, kills the pane, marks the repo `orphaned` with a named diagnostic, and hands off to the task-tracker memory step before dying, rather than leaving an ambiguous or silently-failed worker.

## What must become true here (durable outcome)

Every spawn failure path converges on the same explicit orphaning + evidence-recording sequence, never a silent or ambiguous half-started worker — per the Statement above, which is the operative rule this stage exists to enforce.

