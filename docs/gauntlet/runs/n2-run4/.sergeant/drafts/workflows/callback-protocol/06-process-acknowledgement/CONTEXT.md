# 06-process-acknowledgement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a consumer returns from a callback invocation

**Outcome:** the event's next state is determined by this closed set of outcomes, with every malformed/unexpected response defaulting to pending (never silently ack'd)

**Statement (the operative rule):** `ack` durably suppresses all later callback attempts for that generation; `retry` keeps the event pending with an optional bounded `retry_after_seconds` (0-3600); `reject` records a permanent policy failure without deleting or auto-retrying the event; any timeout, nonzero exit, malformed JSON, wrong version/key, unknown field/status, or oversized output leaves the event pending.

## What must become true here (durable outcome)

The event's next state is determined by this closed set of outcomes, with every malformed/unexpected response defaulting to pending (never silently ack'd) — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0226`: Consumer stderr and output details are never persisted.
- `BU-0776`: A stored callback delivery state's field set and schema version must match exactly, and its status must be one of pending, delivering, acknowledged, or rejected, or it is rejected as unsupported or invalid.
- `BU-0777`: A stored callback delivery state's attempt count and next-attempt time must be non-negative integers, its claim and acknowledgement timestamps must be the correct type or absent, and its last delivery result must be one of a fixed set of outcomes.
- `BU-0791`: A callback invocation is bounded by a configurable timeout (1-120 seconds); exceeding it is treated as its own distinct delivery outcome ('timeout') rather than letting the call hang indefinitely or crash the drain loop.
- `BU-0792`: A non-zero exit from the callback executable is always treated as a delivery failure ('callback_error'), never as an implicit success.
- `BU-0793`: The callback's stdout is size-capped, and any oversized, non-UTF8, non-JSON, or non-dict response is treated as 'invalid_ack' rather than assumed to mean success.
- `BU-0794`: The callback acknowledgement's field set is an exact allowlist (version, idempotency_key, status, retry_after_seconds); any unexpected field invalidates the whole response.
- `BU-0795`: A callback's acknowledgement is only accepted if its version and idempotency key exactly match the event that was sent to it — an ack cannot be misapplied to acknowledge a different event.
- `BU-0796`: A callback acknowledgement's status must be ack, retry, or reject; retry_after_seconds is only meaningful — and only accepted — when status is retry, and must be a bounded non-negative integer when present.
- `BU-0803`: An acknowledged outcome moves a callback event to a terminal 'acknowledged' state; a reject outcome moves it to a terminal 'rejected' state; any other outcome (timeout, callback_error, invalid_ack, retry) returns it to 'pending' with a computed backoff — an event is never left stuck in 'delivering'.

