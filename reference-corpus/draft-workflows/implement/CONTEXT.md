# Implement
Draft workflow package — candidate **W23** `implement` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Implement a piece of work from a spec or ticket set, explicit-invocation-only.

## Trigger

Explicitly invoked to implement a defined piece of work (never auto-loaded).

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-implement-with-tdd` | actor-stage (§6.4, judgment) | Implementation proceeds seam by seam. |
| `20-verify` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Typecheck and focused tests run during implementation; the full suite runs once at the end. |
| `30-review` | actor-stage (§6.4, judgment) | The change is reviewed. |
| `40-commit` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The verified change is committed. |

## Relationships to other workflows

- `10-implement-with-tdd` delegates to **tdd**.
- `30-review` delegates to **code-review**.

## Notes for reviewers

Explicit-invocation-only (BU-P2-051) — this workflow must never be auto-loaded merely because the task looks like implementation; its cross-harness mirror is BU-P3-004.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
