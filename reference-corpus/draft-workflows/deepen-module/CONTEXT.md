# Deepen Module
Draft workflow package — candidate **W25** `deepen-module` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Turn a shallow module into a deep one at a deliberately chosen seam.

## Trigger

A module's interface needs redesign, or a port/adapter decision needs to be made deliberately rather than by default.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-classify-dependencies` | actor-stage (§6.4, judgment) | A four-way classification determines whether a port is needed at all. |
| `10-design-it-twice` | actor-stage (§6.4, judgment) | At least 3 independently generated, structurally different designs, each under a distinct constraint, compared on depth/locality/seam placement, ending in an opinionated recommendation. |
| `20-test-at-new-interface` | actor-stage (§6.4, judgment) | Old shallow-module tests are deleted; new tests assert through the interface only. |

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
