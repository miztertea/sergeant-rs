# To Spec
Draft workflow package — candidate **W31** `to-spec` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Turn a plan/design into a published spec ticket: gathered context, sketched seams, confirmed with the user, published on template.

## Trigger

A design needs to be turned into a spec-shaped ticket before implementation.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-gather-context` | actor-stage (§6.4, judgment) | Context is synthesized, never gathered by interview. |
| `10-sketch-seams` | actor-stage (§6.4, judgment) | The fewest new seams at the highest possible seam, confirmed with the user; a fixed template is published to the tracker with the ready label. |

## Notes for reviewers

**N1 adjudication A4:** the former `20-write-and-publish` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `10-sketch-seams` as a helper invocation. See `provenance.md`'s "Adjudication A4" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
