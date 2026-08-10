# Grill with Docs
Draft workflow package — candidate **W29** `grill-with-docs` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Runs the `grilling` interview while using the domain-modeling discipline to capture ADRs/glossary entries as decisions land.

## Trigger

A plan or design needs interview-style stress-testing that should also produce durable domain artifacts.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-interview-loop` | actor-stage (§6.4, judgment) | One question at a time, waiting for each answer. |
| `10-confirm-understanding` | actor-stage (§6.4, judgment) | An explicit user confirmation gate before any action. |
| `20-capture-decisions` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Decisions landed during the interview are captured as ADRs/glossary entries per domain-modeling conventions. |

## Relationships to other workflows

- `00-interview-loop` delegates to **grilling**.
- `10-confirm-understanding` delegates to **grilling**.

## Notes for reviewers

This is the corpus's cleanest example of workflow composition **without** nesting — representable today by inlining `grilling`'s two stages ahead of the capture step, which is why it does *not* raise an engine gap. Explicit-invocation-only (BU-P3-002).

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
