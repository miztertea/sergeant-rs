# Route Review Findings
Draft workflow package — candidate **W16** `route-review-findings` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Turn independent review output into tracked work and a gate.

## Trigger

A review pass (worker-mission's `30-independent-review`, or code-review) has produced findings.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-parse-and-sanitize` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Raw review output is parsed and sanitized before anything downstream consumes it. |
| `10-retain-artifact` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A sanitized copy is written to durable storage before any external side effect; the failure diagnostic names the retryable next action. |
| `20-route-each` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Each finding is routed with a dedup marker scoped to axis+source+id+parent+branch; a divergent stored body is refused untouched. |
| `30-publish-or-clear-gate` | actor-stage (§6.4, judgment) | The gate is published or cleared only after every finding reached tracked work. |

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
