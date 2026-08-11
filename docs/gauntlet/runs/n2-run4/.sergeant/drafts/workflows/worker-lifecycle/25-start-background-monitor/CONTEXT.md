# 25-start-background-monitor

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** _sgt_background_watch runs on a host without systemd user services

**Outcome:** the call dies immediately with an actionable alternative rather than partially starting a monitor it cannot manage

**Statement (the operative rule):** Starting a managed background monitor fails fast, before touching any state, if the required systemd-run or systemctl tooling is not available, naming the foreground the interactive fleet-watch loop command as the alternative.

## What must become true here (durable outcome)

The call dies immediately with an actionable alternative rather than partially starting a monitor it cannot manage — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0920`: Starting a background monitor is idempotent: if a unit with the deterministic per-task name is already active, its live InvocationID is adopted and the ownership files are refreshed, rather than attempting to start a duplicate unit (which would fail with 'unit already exists').
- `BU-0921`: A monitor's ownership files are written invocation-id-first, then unit-name second, so that a crash between the two writes leaves monitor_unit absent; a later cleanup pass that finds monitor_unit missing silently skips rather than dying on an incomplete/missing invocation id.
- `BU-0922`: After starting a new monitor unit, its InvocationID is read with a bounded retry (up to 20 attempts at 0.1s intervals) because systemd assigns the InvocationID asynchronously after the unit becomes active; if it never appears, the call dies.

