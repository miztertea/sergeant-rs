# Monitor Fleet
Draft workflow package — candidate **W13** `monitor-fleet` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Observe fleet state without mutating it.

## Trigger

An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-snapshot` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A bounded, constant-size, versioned, strictly read-only snapshot; `busy:true` only with a verified witness, otherwise `busy:null`. |
| `10-evaluate-liveness` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Identity plus recent meaningful progress with a defined fallback chain; a stalled live worker records a non-terminal diagnostic, never an automatic kill. |

## Adjudication note (A7)

At N1 adjudication A7 (BH-07), this package's two mutating stages —
`20-reconcile-terminal` and `30-background-watch` — moved to
**reconcile-and-cleanup-fleet**, which already owned fleet mutation and
cleanup. `monitor-fleet` keeps only its strictly read-only outcome: a
bounded snapshot and a liveness evaluation, neither of which writes fleet
state. See `reconcile-and-cleanup-fleet/provenance.md` for where the moved
units landed and `reconcile-and-cleanup-fleet/CONTEXT.md` for the
receiving stage.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
