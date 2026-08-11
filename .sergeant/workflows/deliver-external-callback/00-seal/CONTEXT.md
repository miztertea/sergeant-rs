# 00-seal: seal

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

No cleanup while any event is unacknowledged; sealed history is retired, not deleted.

Trigger (workflow-level): A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

## What must become true here (durable outcome)

No cleanup while any event is unacknowledged; sealed history is retired, not deleted.

## Behavior contract

- **A fleet task's callback events cannot be cleaned up (sealed) while any event for it remains unacknowledged, and once sealed a callback task's event history is marked retired rather than deleted, so no cleanup can silently discard an external system's undelivered or unacknowledged notification.**
  (trigger: fleet state cleanup wants to retire a task's callback history; outcome: cleanup can never destroy evidence of an external notification that has not been proven to have been received)
  — `BU-P6-122`, `reference/sergeant-upstream/bin/sgt-callback` (L861-881)

## Helpers (folded per N1 adjudication A4)

This workflow originally decomposed the durable-callback protocol into four stages (`00-enqueue`, `10-drain-and-retry`, `20-validate-acknowledgement`, `30-seal`). Per N1 adjudication A4 (finding N1-BH-02), `00-enqueue`, `10-drain-and-retry`, and `20-validate-acknowledgement` carried no argument beyond the §6.5 deterministic-machinery boilerplate — none of the three offered an "Additional note" checkpoint argument — so all three demote by default and fold into this stage as helper invocations crossing the seal checkpoint. Their behavior units are preserved here as the machinery this stage's actor invokes before deciding whether the task's callback history may be sealed:

- **Enqueue.** A callback event's identity is derived by hashing its event type together with a caller-supplied source ID, and enqueuing the same (type, source) pair twice is idempotent — it returns the already-recorded event rather than creating a duplicate — so a caller can safely retry enqueuing without risking a duplicate delivery.
  — `BU-P6-120`, `reference/sergeant-upstream/bin/sgt-callback` (L425-441)
- **Drain and retry.** Draining pending callback events claims one event at a time under a lock (marking it 'delivering' with a claim timestamp before releasing the lock to actually invoke the external profile), and a claim that has sat unresolved past a claim timeout is treated as abandoned and reclaimable by a later drain pass, rather than staying stuck forever. A durable external callback event is delivered at-least-once but recorded exactly-once per idempotency key, is retried with exponential backoff bounded by a configurable cap.
  — `BU-P6-121`, `BU-P6-117`, `reference/sergeant-upstream/bin/sgt-callback` (L692-701, L1-2)
- **Validate acknowledgement.** Every terminating callback response (ack, retry, reject) is validated against a strict schema before being trusted, so a malformed or oversized acknowledgement can never be mistaken for success. The Durable Callback Protocol records origin, delivery, and event state per profile-bound callback in a strict, versioned schema (`sergeant.callback-ack/v1`), keyed by an idempotency key.
  — `BU-P6-117`, `BU-P7-067`, `reference/sergeant-upstream/bin/sgt-callback` (L1-2), `reference/sergeant-upstream/tests/sgt-callback-test.sh` (lines 27-73)

## Fixer note (A4 structural exception)

Unlike every other package in this sweep, none of this package's four originally-extracted stages ever carried a "Judgment required" (§6.4) heading — the whole procedure, start to finish, is deterministic queue/retry/validation machinery with no discretionary actor decision (no alternative to weigh, nothing to ask the user). A4's instruction to fold demoted machinery stages "into its adjacent judgment-bearing stage" presumes such a stage exists; here none does. Rather than invent a judgment argument the evidence does not support, this stage is kept as the sole structural host (renamed from `30-seal` to `00-seal` since it is now the workflow's only stage) purely because `docs/icm/convention.md` requires a materializable workflow to have at least one actor-stage directory, and `30-seal`'s terminal gate (never clean up unacknowledged events) is the closest thing this package has to a durable checkpoint. Confidence in this specific stage-vs-helper placement is **low**; the package's own engine-gap **G3** note already identifies the whole area as machinery pressure this milestone's engine cannot yet own natively, which is consistent with the honest read that this may not be a "workflow" with actor judgment at all, only a queue the engine should eventually own directly.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
