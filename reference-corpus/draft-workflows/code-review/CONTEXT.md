# Code Review
Draft workflow package — candidate **W24** `code-review` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Review a diff on two parallel, non-contaminating axes — Standards and Spec — via isolated sub-reviews, reported side by side.

## Trigger

A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-pin-fixed-point` | actor-stage (§6.4, judgment) | The fixed point resolves and the diff is non-empty, or this fails here rather than inside a sub-review. |
| `10-identify-spec-source` | actor-stage (§6.4, judgment) | The spec source is identified via a fixed priority order ending in asking the user. |
| `20-parallel-review-standards` | actor-stage (§6.4, judgment) | An isolated review against the repository's documented coding standards. |
| `30-parallel-review-spec` | actor-stage (§6.4, judgment) | An isolated review against the identified spec source. |
| `40-aggregate` | actor-stage (§6.4, judgment) | The two axes are reported separately, never merged or reranked. |

## Notes for reviewers

The two-axis separation is the durable design point (BU-P2-018), not the sub-agent mechanism that happens to isolate the two reviews from each other.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
