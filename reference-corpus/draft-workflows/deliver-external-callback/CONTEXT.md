# Deliver External Callback
Draft workflow package — candidate **W17** `deliver-external-callback` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Durable at-least-once notification to a registered external consumer.

## Trigger

A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-enqueue` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Identity is hashed from (type, source); a repeat enqueue returns the existing event rather than duplicating it. |
| `10-drain-and-retry` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | One event claimed at a time under a lock; stale claims are reclaimable; backoff is bounded and exponential. |
| `20-validate-acknowledgement` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A strict versioned schema; malformed or oversized responses never count as success. |
| `30-seal` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | No cleanup while any event is unacknowledged; sealed history is retired, not deleted. |

## Notes for reviewers

Raises engine-gap **G3** (durable outbound notification queue) — survives with a required amendment narrowing the runtime-owned core to an acknowledgement gate on terminal Work cleanup, not the whole delivery queue. See `reference-corpus/synthesis.md` §5.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
