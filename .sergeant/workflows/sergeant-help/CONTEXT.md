# Sergeant Help
Draft workflow package — candidate **W4** `sergeant-help` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Answer a Sergeant usage/setup/troubleshooting question from repository-owned documentation with an explicit precedence order, read-only, never inventing behavior.

## Trigger

The user asks what Sergeant is, how to install/configure/use it, where skills come from, or how to diagnose a Sergeant error.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-classify-and-locate` | actor-stage (§6.4, judgment) | The question is bound to one primary document, which is read before any broad search; a missing primary document stops the run with its expected path. |
| `10-resolve-source-conflicts` | actor-stage (§6.4, judgment) | Where sources disagree, the answer follows the fixed precedence and the mismatch is reported as tracked work. |
| `20-answer-or-hand-off` | actor-stage (§6.4, judgment) | Either a fixed-format answer with command, preconditions, evidence and doc links, or an explicit hand-off to the owning procedure. |

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
