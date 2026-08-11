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
| `00-seal` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md; A4 structural exception, see below) | No cleanup while any event is unacknowledged; sealed history is retired, not deleted. |

## Notes for reviewers

**N1 adjudication A4 (finding N1-BH-02).** This package originally decomposed the callback protocol into four stages (`00-enqueue`, `10-drain-and-retry`, `20-validate-acknowledgement`, `30-seal`). None carried an argument beyond the §6.5 deterministic-machinery boilerplate, so all four demote by default. Unlike other packages in this sweep, none of the four ever carried a "Judgment required" heading — this package has no discretionary actor decision anywhere in it. `00-enqueue`, `10-drain-and-retry`, and `20-validate-acknowledgement` fold into `00-seal` (renamed from `30-seal`, now the workflow's sole stage) as helper invocations; see that stage's "Fixer note (A4 structural exception)" for why it is kept as the structural host rather than the fold producing a stageless package. The behavior units survive as helper material — see `00-seal/CONTEXT.md` and `provenance.md`.

Raises engine-gap **G3** (durable outbound notification queue) — survives with a required amendment narrowing the runtime-owned core to an acknowledgement gate on terminal Work cleanup, not the whole delivery queue. See `reference-corpus/synthesis.md` §5.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
