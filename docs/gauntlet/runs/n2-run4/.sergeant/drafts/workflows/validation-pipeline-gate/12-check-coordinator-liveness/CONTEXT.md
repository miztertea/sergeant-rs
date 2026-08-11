# 12-check-coordinator-liveness

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the validation worker polls whether its coordinator is still alive

**Outcome:** a reused PID is never mistaken for the still-live original coordinator

**Statement (the operative rule):** The validation worker treats the coordinator as alive only if the validation-launch lock file exists (and is not a symlink), its recorded pid responds to kill -0, and that pid's current process start time still matches what was recorded at lock-acquisition time.

## What must become true here (durable outcome)

A reused PID is never mistaken for the still-live original coordinator — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0395`: The validation worker waits for the coordinator's validation-launch lock to be released before running the validation pipeline, but if the coordinator process is found no longer alive while the lock file still exists, it fails rather than proceeding, unless the lock has genuinely already been removed by the time it checks.

