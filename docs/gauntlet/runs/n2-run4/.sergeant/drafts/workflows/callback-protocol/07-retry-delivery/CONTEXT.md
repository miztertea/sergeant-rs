# 07-retry-delivery

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a callback event delivery is retried

**Outcome:** delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery

**Statement (the operative rule):** Each callback event is claimed before invocation, a stale `delivering` claim becomes eligible again after 60 seconds by default, failed attempts use exponential backoff (5-300 seconds by default), and each drain processes a bounded number of distinct events.

## What must become true here (durable outcome)

Delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0228`: After repairing a permanent consumer policy/configuration failure, an operator can requeue a retained event with the callback-delivery step without changing its idempotency key.
- `BU-0797`: A callback-requested retry delay (retry_after_seconds) is honored as given; absent that, the retry delay is computed as an exponential backoff from a configurable base, capped at a configurable maximum.
- `BU-0798`: Before a callback event is delivered, the callback-delivery step durably records it as 'delivering' (with an incremented attempt count and claim timestamp) while holding the task lock, then releases the lock before actually invoking the callback subprocess.
- `BU-0799`: A callback event already claimed as 'delivering' is only eligible to be reclaimed for a fresh attempt once its claim has been held longer than a configurable claim timeout — a still-fresh in-flight claim is left alone.
- `BU-0800`: Callback events in a terminal state (acknowledged or rejected) are never revisited by drain.
- `BU-0801`: Callback events not yet due (next_attempt_at in the future) are skipped for the current drain pass.
- `BU-0802`: After a callback invocation returns, the callback-delivery step only writes the delivery outcome back if the event's stored state still exactly matches the attempt count and claim timestamp this same attempt itself set — if the state has since changed, the outcome is discarded rather than overwriting whatever changed it.
- `BU-0804`: Draining all tasks does not abort the whole sweep on one task's failure — every fleet task with a registered callback origin is attempted, and failures are collected and reported together at the end.
- `BU-0807`: A callback event that has already reached the acknowledged terminal state cannot be retried.
- `BU-0808`: Retrying a matched callback event resets its state to immediately eligible (pending, next_attempt_at=0, claimed_at=None), so the next drain attempts it right away rather than waiting for its previously scheduled backoff.
- `BU-0809`: retry_event fails if the given idempotency key does not match any known event for the task.

