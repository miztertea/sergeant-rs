# 07-enter-waiting-state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a worker needs to wait on an external condition

**Outcome:** the wait is represented durably (wake-condition file + waiting status) rather than a live sleep loop, permitting clean exit

**Statement (the operative rule):** `waiting` is used instead of sleep loops for CI checks, dependency completion, and time-based delays: the worker writes `.sergeant-wake-condition`, sets `.sergeant-status=waiting`, and may exit cleanly after its durable handoff.

## What must become true here (durable outcome)

The wait is represented durably (wake-condition file + waiting status) rather than a live sleep loop, permitting clean exit — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0307`: `.sergeant-gate-generation` is a monotonic integer for waiting gates: before every new `needs_input` or `blocked` publication, the worker increments and persists the generation before writing the waiting status and message, so a repeated blocker message only counts as a new gate when the generation actually advances.
- `BU-0308`: Only the allowlisted wake-condition field names and alphanumeric-safe values are accepted in `.sergeant-wake-condition`; arbitrary shell commands, prompt bodies, response text, tokens, or secrets are never persisted there.

