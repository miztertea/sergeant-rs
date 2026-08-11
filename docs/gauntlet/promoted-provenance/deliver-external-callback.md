# Provenance — Deliver External Callback

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W17** `deliver-external-callback`.

## Stages

### `00-seal`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-122` | A fleet task's callback events cannot be cleaned up (sealed) while any event for it remains unacknowledged, and once sealed a callback task's event history is marked retired rather than deleted, so no cleanup can silently discard an external system's undelivered or unacknowledged notification. | `reference/sergeant-upstream/bin/sgt-callback` (L861-881) |
| `BU-P6-120` (folded helper: enqueue) | A callback event's identity is derived by hashing its event type together with a caller-supplied source ID, and enqueuing the same (type, source) pair twice is idempotent — it returns the already-recorded event rather than creating a duplicate — so a caller can safely retry enqueuing without risking a duplicate delivery. | `reference/sergeant-upstream/bin/sgt-callback` (L425-441) |
| `BU-P6-121` (folded helper: drain and retry) | Draining pending callback events claims one event at a time under a lock (marking it 'delivering' with a claim timestamp before releasing the lock to actually invoke the external profile), and a claim that has sat unresolved past a claim timeout is treated as abandoned and reclaimable by a later drain pass, rather than staying stuck forever. | `reference/sergeant-upstream/bin/sgt-callback` (L692-701) |
| `BU-P6-117` (folded helper: drain/retry + validate) | A durable external callback event is delivered at-least-once but recorded exactly-once per idempotency key, is retried with exponential backoff bounded by a configurable cap, and every terminating callback response (ack, retry, reject) is validated against a strict schema before being trusted, so a malformed or oversized acknowledgement can never be mistaken for success. | `reference/sergeant-upstream/bin/sgt-callback` (L1-2) |
| `BU-P7-067` (folded helper: validate acknowledgement) | The Durable Callback Protocol records origin, delivery, and event state per profile-bound callback in a strict, versioned schema (`sergeant.callback-ack/v1`), keyed by an idempotency key, so registration writes only the versioned strict-origin contract and every emitted event type is independently observable and classifiable. | `reference/sergeant-upstream/tests/sgt-callback-test.sh` (lines 27-73) |

## Notes

**Synthesis notes:** Raises engine-gap **G3** (durable outbound notification queue) — survives with a required amendment narrowing the runtime-owned core to an acknowledgement gate on terminal Work cleanup, not the whole delivery queue. See `reference-corpus/synthesis.md` §5.

## Adjudication A4 (N1-BH-02 sweep)

Original stages: `00-enqueue`, `10-drain-and-retry`, `20-validate-acknowledgement`, `30-seal`. All four carried only the §6.5 deterministic-machinery boilerplate as their extraction justification — none had an "Additional note" checkpoint argument — so per A4's default rule all four demote.

This package is the sweep's structural edge case: unlike every sibling package, none of its four stages was ever classified "Judgment required" (§6.4) — the whole procedure is deterministic queue/retry/validation machinery with no actor discretion anywhere. A4's fold instruction ("into its adjacent judgment-bearing stage") presumes a judgment-bearing stage exists to receive the fold; here none does.

**Decision:** `00-enqueue`, `10-drain-and-retry`, and `20-validate-acknowledgement` are demoted and folded as helper invocations into `30-seal`, which is retained and renamed `00-seal` (now the workflow's sole stage) as the structural host required by `docs/icm/convention.md` (a materializable workflow needs at least one actor-stage directory with a `CONTEXT.md`). This is recorded as a **low-confidence** placement, not a discovered judgment argument: `30-seal`'s "never clean up an unacknowledged event" gate is simply the closest thing this package has to a durable checkpoint. The behavior units themselves are not deleted — see `00-seal/CONTEXT.md`'s "Helpers (folded per N1 adjudication A4)" section, which lists all four original units.

This finding is consistent with, and does not contradict, the package's own engine-gap **G3** note: the honest reading is that this package may not be a "workflow" with actor judgment at all, only a queue the engine should eventually own directly (proposal §12.3's `execute`-stage kind, once it exists).

## Promotion note (`docs/icm/promotion-spec-2026-08-11.md`)

`00-seal`, this package's true (and only) closing stage, declares a `promote` output disposition with no finalize step — one of the 30 of 34 N1 packages in that shape, not one of the 3 (`drain-fleet`, `respond-to-worker`, `to-spec`) that name one. Recorded here per the spec's finalize-gap rule rather than silently promoted; disposition on whether this package needs a finalize step is left to human review at merge time, not applied mechanically by this curation act.

**NEEDS-JUDGMENT resolution (§5):** this package's classification turns on the A4 low-confidence stage-vs-helper placement recorded above, not on an unverified delegation target (it names none). Curation does not reclassify that placement or add a "Judgment required" section to it — both are forbidden re-adjudication under §2. The placement is packaged verbatim; §3's engine-acceptance gate exercises `00-seal` as an ordinary single-stage completion (this package has no `needs_input` dependency, unlike `grilling`/`sergeant-setup`'s G5 stages), so a clean gate pass here is full mechanical confirmation, not the partial one those two packages get.
