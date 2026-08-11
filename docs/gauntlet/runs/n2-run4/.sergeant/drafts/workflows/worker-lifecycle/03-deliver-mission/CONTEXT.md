# 03-deliver-mission

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a worker's mission/brief is being delivered at launch

**Outcome:** delivery is exactly-once-safe across TUI startup delay or coordinator crash, and never exposes the mission body via process args

**Statement (the operative rule):** A worker-owned loop retries only a fixed ID-bearing terminal nudge until the agent acknowledges that ID before acting, so delayed TUI startup and coordinator crashes do not lose or duplicate the mission, and no body appears in process arguments.

## What must become true here (durable outcome)

Delivery is exactly-once-safe across TUI startup delay or coordinator crash, and never exposes the mission body via process args — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0305`: A worker follows a notification instruction exactly once per token; repeated nudges carrying the same token are treated as retries, not new work.
- `BU-0911`: Notification delivery-confirmed (the nudge reached the exact target pane and was accepted) is a distinct, separately durable state from action-completion (the agent actually acted), recorded under separate artifacts (targets/<nonce>/handshake_complete vs. targets/<nonce>/completed), because previously conflating the two let a caller believe a turn had settled when nothing had actually published completion.
- `BU-0912`: While waiting for notification delivery, the target pane's live identity is re-verified against the expected identity on every polling iteration, not only once at the start, before any delivered/accepted marker for that pane is trusted.
- `BU-0913`: Waiting for worker-notification delivery is bounded by a configurable timeout (SGT_NOTIFICATION_ACK_TIMEOUT, default 60 seconds); once the bound is exceeded the wait returns failure rather than fabricating a success.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0907`: A notification id must match a strict alphanumeric/dot/hyphen/underscore charset before any notification-publish state is touched; anything else is rejected immediately.
- `BU-0908`: A notification's durable record is only overwritten when its content actually differs from what is already on disk (compared byte-for-byte before publishing); when it does need to change, the new content is written to a temp file and atomically renamed into place.

