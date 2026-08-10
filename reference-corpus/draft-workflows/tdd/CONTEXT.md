# TDD
Draft workflow package — candidate **W22** `tdd` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Test-driven development for one confirmed seam at a time: red, green, one minimal implementation.

## Trigger

A feature or bug fix is being implemented test-first.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-agree-seams` | actor-stage (§6.4, judgment) | Seams are written down and confirmed with the user; no test is written at an unconfirmed seam. |
| `10-red-green-cycle` | actor-stage (§6.4, judgment) | One seam, one test, one minimal implementation, vertical slices only. |

## Notes for reviewers

Refactoring is explicitly *not* a stage of this workflow (BU-P2-116) — it hands off to `code-review`/deepen-module discipline instead. The bulk of the `tdd` source is reference guidance, not procedure (16 units land in the `test-quality` shared context, not in this workflow).

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
