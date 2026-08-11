# 10-reset-retryable-state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the project-validation step is retried after a prior validation exit

**Outcome:** a retry can never reset state while genuinely live validation processes, primary or unverified detached descendants, remain running

**Statement (the operative rule):** Resetting retryable validation state after a prior exit refuses to proceed if the recorded owner PID is still alive with the same recorded start time and process group (validation processes are still genuinely running), or if pgrep finds any live descendant process still in the recorded process group even after the primary PID is confirmed dead.

## What must become true here (durable outcome)

A retry can never reset state while genuinely live validation processes, primary or unverified detached descendants, remain running — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0379`: Before removing the isolated validation code snapshot during a retry reset, the project-validation step requires lsof to be installed and uses it to verify no process still has any file inside the snapshot open; any such process aborts the reset with the offending PIDs named.

