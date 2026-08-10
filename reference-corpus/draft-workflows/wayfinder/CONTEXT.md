# Wayfinder
Draft workflow package — candidate **W33** `wayfinder` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Map an unfamiliar frontier of a codebase or problem space, ticket-izing decisions and resolving them one at a time.

## Trigger

A destination is named that requires mapping fog before it can be reached.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-name-destination` | actor-stage (§6.4, judgment) | The destination is named via a grilling/domain-modeling session; scope is settled first. |
| `10-map-frontier` | actor-stage (§6.4, judgment) | Breadth-first mapping; stop and do not create a map if no fog exists. |
| `20-create-tickets` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Specifiable decisions become child issues first; blocking edges are wired in a second pass. |
| `30-resolve-one` | actor-stage (§6.4, judgment) | Claim, resolve by type, record the answer as a resolution and a one-line pointer; at most one non-research ticket per session. |
| `40-regraduate-fog` | actor-stage (§6.4, judgment) | Remaining fog is re-evaluated; the run loops back to `10-map-frontier` if fog remains. |

## Relationships to other workflows

- `00-name-destination` delegates to **grilling**.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
