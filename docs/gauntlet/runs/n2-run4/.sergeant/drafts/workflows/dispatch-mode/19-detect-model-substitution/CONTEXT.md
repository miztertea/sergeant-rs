# 19-detect-model-substitution

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the claude attach invocation has exited and a model was pinned for this session

**Outcome:** a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully

**Statement (the operative rule):** After the main Claude invocation exits, if a model was pinned, the worker scans the session transcript for the substitution warning line ("is restricted by your organization's settings... Using <Y> instead") and, if found, records a durable model_substitution_warning in fleet state and prints a WARNING — since neither the pre-flight regex nor the liveness check can catch a valid-but-unentitled model alias that Claude silently substitutes without any failed state or nonzero exit code.

## What must become true here (durable outcome)

A silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0368`: The model-substitution warning is written to its own dedicated file rather than $REPO_STATE/diagnostic, because _finish unconditionally clears the shared diagnostic file on every non-orphaned exit (including this successful-mission path), which would otherwise erase the warning before anything could read it.

