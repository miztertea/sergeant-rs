# Wake and Resume
Draft workflow package — candidate **W14** `wake-and-resume` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Resume a waiting worker when its durable condition is met.

## Trigger

A worker is in the `waiting` state with a recorded wake condition.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-evaluate` | stage (§6.3, deterministic-machinery candidate — kept, see Adjudication note) | Validate condition, evaluate one of six typed condition kinds (external checks bound to the worker's own recorded remote), classify the outcome, and resume the worker on a met outcome. |

## Adjudication note (A4)

N1 adjudication A4 (BH-02) applied the generic de-staging sweep: `00-validate-condition`, `20-classify-outcome`, and `30-resume` carried no argument beyond the §6.5 boilerplate and folded into `10-evaluate` as ordered helper invocations. `10-evaluate` itself was kept, but not for a judgment reason — its "Additional note" is the direct source of accepted engine-gap **G1** (this stage's periodic, processless re-evaluation is exactly what no lower rung can own), a categorical argument distinct from and stronger than §6.3's implementation-swap test. Stage count dropped from 4 to 1; no behavior unit was deleted — see `provenance.md`'s "Adjudication A4" section and `10-evaluate/CONTEXT.md`'s "Helper invocations" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
