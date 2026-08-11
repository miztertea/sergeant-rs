# 19-relaunch-superseded-worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a relaunch is about to start a new worker supervisor pane

**Outcome:** two Claude processes are never running concurrently against the same worktree

**Statement (the operative rule):** Before dispatching a replacement worker, any live Claude background session left by the superseded worker is stopped first — the stop always happens before the new dispatch, never after.

## What must become true here (durable outcome)

Two Claude processes are never running concurrently against the same worktree — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0428`: If tmux fails to create the relaunch window/pane, the worker response-delivery step records the worker as orphaned (with the failure reason in the diagnostic) and releases the drain admission lock before dying.
- `BU-0429`: The relaunched pane's identity is published and a notification target is created for it; if that creation detects a race, the just-started background session is stopped, the new pane is killed, the worker is recorded orphaned, and the worker response-delivery step dies.
- `BU-0430`: A superseded live pane is killed only after the replacement pane is fully live, its identity published, and its notification target created.
- `BU-0431`: If the relaunched worker's pane does not acknowledge its delivery notification within the bound, the background session and pane are torn down, the worker is recorded orphaned, and the worker response-delivery step dies rather than declaring the relaunch successful.
- `BU-0432`: On a fully successful relaunch, the worker response-delivery step clears both the drain_held and response_delivery_unacked markers.
- `BU-0494`: Before launching the replacement worker pane, any live Claude background session left by the stalled worker is stopped first, always before the new dispatch, never after.
- `BU-0495`: The replacement pane is launched before the stalled pane is ever killed; if the launch fails, the original stalled pane is left completely untouched and recovery escalates to needs_input for investigation.
- `BU-0496`: New pane identity publication and notification-target creation are validated before anything else proceeds; on any failure the new pane is killed, the pane record is restored to the original stalled pane, and recovery escalates — the worker is never left pointing at a pane that was never confirmed live.
- `BU-0497`: The old stalled pane is only killed once the replacement pane is fully launched, identity-confirmed, and holds an active notification target.
- `BU-0498`: If the relaunched worker's pane fails to acknowledge the recovery notification within the bound, its background session and pane are torn down and recovery escalates to needs_input rather than being declared successful.
- `BU-0567`: A recorded Claude background session is only treated as provably live when its queried state is exactly 'working' or 'blocked'; a missing id, an unresolvable binary, jq being unavailable, or any other state is treated as 'not provably live', which is what authorizes proceeding (e.g. dispatching a replacement worker) rather than blocking on an unconfirmable session.

