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
| `00-validate-condition` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A strict field/value allowlist is enforced — no dash-leading values, secret-shaped names screened — before evaluation. |
| `10-evaluate` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | One of six typed condition kinds is evaluated; external checks bind to the worker's own recorded remote. |
| `20-classify-outcome` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Outcome is classified met / unmet / permanently-unsatisfiable→escalate / deadline→failed. |
| `30-resume` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The worker is resumed on a met outcome. |

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
