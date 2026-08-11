# 01-resume-model-pin-reverification

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a worker is resumed or recovered

**Outcome:** the original model pin is honored exactly on resume, and an unhonorable tuple fails terminally instead of silently substituting a default

**Statement (the operative rule):** A resumed or recovered worker reads the same fleet record and inherits the same model pin; a worker handed a tuple its harness cannot honor fails terminally rather than falling back to the ambient default.

## What must become true here (durable outcome)

The original model pin is honored exactly on resume, and an unhonorable tuple fails terminally instead of silently substituting a default — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0344`: The pinned model tuple lives in durable fleet state (not the ambient environment) so that a worker resumed by the worker response-delivery step, the stalled-worker recovery step, or the wake-condition step runs the exact same pinned model as the original dispatch; a tuple the harness cannot honor is a terminal launch failure rather than silently falling back to an ambient default model.

