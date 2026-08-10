# Provenance — Deliver External Callback

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W17** `deliver-external-callback`.

## Stages

### `00-enqueue`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-120` | A callback event's identity is derived by hashing its event type together with a caller-supplied source ID, and enqueuing the same (type, source) pair twice is idempotent — it returns the already-recorded event rather than creating a duplicate — so a caller can safely retry enqueuing without risking a duplicate delivery. | `reference/sergeant-upstream/bin/sgt-callback` (L425-441) |

### `10-drain-and-retry`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-121` | Draining pending callback events claims one event at a time under a lock (marking it 'delivering' with a claim timestamp before releasing the lock to actually invoke the external profile), and a claim that has sat unresolved past a claim timeout is treated as abandoned and reclaimable by a later drain pass, rather than staying stuck forever. | `reference/sergeant-upstream/bin/sgt-callback` (L692-701) |
| `BU-P6-117` | A durable external callback event is delivered at-least-once but recorded exactly-once per idempotency key, is retried with exponential backoff bounded by a configurable cap, and every terminating callback response (ack, retry, reject) is validated against a strict schema before being trusted, so a malformed or oversized acknowledgement can never be mistaken for success. | `reference/sergeant-upstream/bin/sgt-callback` (L1-2) |

### `20-validate-acknowledgement`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-117` | A durable external callback event is delivered at-least-once but recorded exactly-once per idempotency key, is retried with exponential backoff bounded by a configurable cap, and every terminating callback response (ack, retry, reject) is validated against a strict schema before being trusted, so a malformed or oversized acknowledgement can never be mistaken for success. | `reference/sergeant-upstream/bin/sgt-callback` (L1-2) |
| `BU-P7-067` | The Durable Callback Protocol records origin, delivery, and event state per profile-bound callback in a strict, versioned schema (`sergeant.callback-ack/v1`), keyed by an idempotency key, so registration writes only the versioned strict-origin contract and every emitted event type is independently observable and classifiable. | `reference/sergeant-upstream/tests/sgt-callback-test.sh` (lines 27-73) |

### `30-seal`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-122` | A fleet task's callback events cannot be cleaned up (sealed) while any event for it remains unacknowledged, and once sealed a callback task's event history is marked retired rather than deleted, so no cleanup can silently discard an external system's undelivered or unacknowledged notification. | `reference/sergeant-upstream/bin/sgt-callback` (L861-881) |

## Notes

**Synthesis notes:** Raises engine-gap **G3** (durable outbound notification queue) — survives with a required amendment narrowing the runtime-owned core to an acknowledgement gate on terminal Work cleanup, not the whole delivery queue. See `reference-corpus/synthesis.md` §5.

