# 14-monitor-active-run

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the run reaches a gate or a long-running step

**Outcome:** the run only advances through an explicit pipeline-automation tool, never spontaneously

**Statement (the operative rule):** A long-running call is working, not stalled, and may be backgrounded if the harness needs to, but the run never advances past a gate on its own; the agent must read every return, respond on a `gate:`, and loop until an `outcome:` is reached, never idle-waiting for the run to move forward by itself.

## What must become true here (durable outcome)

The run only advances through an explicit pipeline-automation tool, never spontaneously — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1220`: The pipeline-automation tool and every pipeline-automation tool block synchronously, and the review, test, and CI steps can each take several minutes, so a single call may not return for a while; this is normal, and the agent must allow a long timeout and not cancel or re-issue the command because it seems slow.
- `BU-1221`: To check progress without disturbing the run, the agent uses the validation pipeline from a separate call rather than cancelling or re-issuing the blocking call.
- `BU-1223`: The `awaiting_agent: parked <duration>` field appearing under a run in status output is observability only: it does not change gate resolution, does not auto-resume the run, and does not make `--yes` the default.
- `BU-1225`: If `last_activity` is prefixed `quiet`, no step log or native-agent lifecycle activity has arrived for longer than `step_quiet_warning`; this is a liveness clue only, not permission to cancel, rerun, or edit the worktree.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-1224`: While a step is actively `running` or `fixing`, the pipeline-automation tool may include an `active_steps` table with `active_for`, `last_activity`, a native `agent_pid` when a subprocess agent is running, and the current round (e.g. `round 1`, `auto-fix 1/3`, `fix 2`).

