# To Tickets
Draft workflow package — candidate **W32** `to-tickets` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Break a plan, spec, investigation, findings register, PR, or conversation into dependency-aware tracer-bullet work.

## Trigger

The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-load-project-context` | actor-stage (§6.4, judgment) | Project context is loaded. |
| `10-extract-decisions-and-unknowns` | actor-stage (§6.4, judgment) | An investigation ticket is created only for a genuinely blocking unknown, naming the exact artifact it must produce. |
| `20-confirm-breakdown` | actor-stage (§6.4, judgment) | Granularity, ownership and blocking edges are confirmed unless immediate publication was requested. |
| `30-publish` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | New tickets stay open; cross-repo blockers recorded as counterpart ids plus merge order. |
| `40-report-frontier` | actor-stage (§6.4, judgment) | One worker per owning repo is the default; reporting is not authorization to dispatch. |

## Relationships to other workflows

- `00-load-project-context` delegates to **load-project**.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
