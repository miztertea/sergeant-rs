# 14-seal-before-deletion

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the fleet cleanup step is about to delete fleet state for a task with callback events

**Outcome:** a re-verified, locked, sealed check closes the race window between checking and actually deleting

**Statement (the operative rule):** Immediately before fleet deletion, the fleet cleanup step takes the callback lock, verifies the acknowledgement condition again, and writes a terminal seal that rejects new event generations and closes the acknowledgement-check/deletion race.

## What must become true here (durable outcome)

A re-verified, locked, sealed check closes the race window between checking and actually deleting — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0233`: If cleanup fails after sealing and the fleet must resume, an operator removes only the seal with the callback-delivery step.
- `BU-0676`: Before deleting the task's fleet state, the fleet cleanup step seals the task's callback origin, only if an origin.json marker is present, so callback finalization happens while the evidence it depends on still exists.
- `BU-0677`: The fleet cleanup step writes a wiki activity log entry recording the task's final status and result immediately before deleting fleet state, because fleet state is the only place that status and result live; waiting any later would lose the ability to report them.
- `BU-0779`: A callback event cannot be enqueued for a task whose callback directory has been sealed for cleanup.
- `BU-0806`: A callback event cannot be retried inside a task whose callback directory has been sealed for cleanup.
- `BU-0810`: A task's callback directory cannot be sealed for cleanup while any of its callback events remains unacknowledged.
- `BU-0811`: Sealing a task's callback directory is idempotent given an existing valid seal, but an existing seal with unexpected content is treated as an error rather than silently accepted or overwritten.
- `BU-0812`: Unsealing a task validates the existing seal's content before removing it — an unrecognized seal value is rejected rather than blindly removed — and unsealing a task that is not currently sealed is a no-op.

