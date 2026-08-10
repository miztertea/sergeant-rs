# 20-validate-acknowledgement: validate acknowledgement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-drain-and-retry/output/README.md | L4 | upstream artifact produced by `10-drain-and-retry` |

## Purpose

A strict versioned schema; malformed or oversized responses never count as success.

Trigger (workflow-level): A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

## What must become true here (durable outcome)

A strict versioned schema; malformed or oversized responses never count as success.

## Behavior contract

- **A durable external callback event is delivered at-least-once but recorded exactly-once per idempotency key, is retried with exponential backoff bounded by a configurable cap, and every terminating callback response (ack, retry, reject) is validated against a strict schema before being trusted, so a malformed or oversized acknowledgement can never be mistaken for success.**
  (trigger: a fleet task reaches needs_input, blocked, failed, or done, and an external callback origin is registered; outcome: an external system is durably, exactly-once notified of a fleet task's terminal or escalation events, with a documented retry/backoff contract and no ambiguity about what counts as acknowledged)
  — `BU-P6-117`, `reference/sergeant-upstream/bin/sgt-callback` (L1-2)
- **The Durable Callback Protocol records origin, delivery, and event state per profile-bound callback in a strict, versioned schema (`sergeant.callback-ack/v1`), keyed by an idempotency key, so registration writes only the versioned strict-origin contract and every emitted event type is independently observable and classifiable.**
  (trigger: a fleet task registers a callback profile and later emits lifecycle events; outcome: callback delivery has a durable return path independent of the coordinator's own liveness, versioned so consumers can rely on a stable schema, and idempotency-keyed so retried deliveries are safely deduplicated)
  — `BU-P7-067`, `reference/sergeant-upstream/tests/sgt-callback-test.sh` (lines 27-73)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
