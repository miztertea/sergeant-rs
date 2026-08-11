# Reconcile and Cleanup Fleet
Draft workflow package — candidate **W15** `reconcile-and-cleanup-fleet` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Retire a completed task's surfaces and state.

## Trigger

A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-require-terminal` | actor-stage (§6.4, judgment) | Every targeted repo is safely terminal and the owning task is verifiably closed; ownership re-verified, handshakes acknowledged and sealed, each surface removed, whole-task state retired only once every repo is done. |

## Adjudication notes (A4, A7)

At N1 adjudication A7 (BH-07), this package received two mutating stages
moved from `monitor-fleet` — `20-reconcile-terminal` and
`30-background-watch` — because they mutate fleet state and this package
already owns fleet mutation and cleanup.

At N1 adjudication A4 (BH-02), the generic de-staging sweep then applied:
every stage here whose only ladder justification was the §6.5
deterministic-machinery boilerplate (`10-verify-ownership`,
`20-verify-handshakes`, `30-remove-surface`, `40-retire-state`, plus the
two stages received from `monitor-fleet`) folded into the package's sole
judgment-bearing stage, `00-require-terminal`, as ordered helper
invocations. None of the six carried an "Additional note" checkpoint
argument that survived §6.3's reimplementation test — `30-background-watch`
in particular argued its own demotion at extraction ("closer to a
deterministic helper... than a procedural checkpoint"). See
`00-require-terminal/CONTEXT.md` for the full folded content and
`provenance.md`'s "Adjudication A4" section for the per-unit disposition.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
