# 10-drain-and-retry: drain and retry

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-enqueue/output/README.md | L4 | upstream artifact produced by `00-enqueue` |

## Purpose

One event claimed at a time under a lock; stale claims are reclaimable; backoff is bounded and exponential.

Trigger (workflow-level): A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

## What must become true here (durable outcome)

One event claimed at a time under a lock; stale claims are reclaimable; backoff is bounded and exponential.

## Behavior contract

- **Draining pending callback events claims one event at a time under a lock (marking it 'delivering' with a claim timestamp before releasing the lock to actually invoke the external profile), and a claim that has sat unresolved past a claim timeout is treated as abandoned and reclaimable by a later drain pass, rather than staying stuck forever.**
  (trigger: a callback drain pass is selecting the next event to attempt delivery for; outcome: a crashed or stuck delivery attempt is automatically reclaimed by a later drain pass instead of permanently blocking that event)
  — `BU-P6-121`, `reference/sergeant-upstream/bin/sgt-callback` (L692-701)
- **A durable external callback event is delivered at-least-once but recorded exactly-once per idempotency key, is retried with exponential backoff bounded by a configurable cap, and every terminating callback response (ack, retry, reject) is validated against a strict schema before being trusted, so a malformed or oversized acknowledgement can never be mistaken for success.**
  (trigger: a fleet task reaches needs_input, blocked, failed, or done, and an external callback origin is registered; outcome: an external system is durably, exactly-once notified of a fleet task's terminal or escalation events, with a documented retry/backoff contract and no ambiguity about what counts as acknowledged)
  — `BU-P6-117`, `reference/sergeant-upstream/bin/sgt-callback` (L1-2)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
