# 26-stop-background-monitor

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** _sgt_stop_background_monitor finds a registered unit name but no stored invocation id

**Outcome:** the call refuses to stop anything and reports the inconsistency instead of guessing

**Statement (the operative rule):** Stopping a background monitor dies rather than proceeding when a monitor unit is registered but its stored invocation id is missing, because the identity needed to safely target the stop cannot be established.

## What must become true here (durable outcome)

The call refuses to stop anything and reports the inconsistency instead of guessing — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0924`: Stopping a background monitor refuses to act, and dies with a diagnostic, if the unit's current live InvocationID differs from the one stored when the monitor was started, because a different process instance now holds that unit name (a TOCTOU/unit-reuse hazard, analogous to PID reuse).

