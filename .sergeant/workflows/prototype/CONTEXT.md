# Prototype
Draft workflow package — candidate **W21** `prototype` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Build a throwaway prototype to answer a design question, branching between logic and UI questions.

## Trigger

The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-select-branch` | actor-stage (§6.4, judgment) | Which question type (logic vs. UI) is decided; a heuristic fallback is recorded when the user is unreachable. |
| `10-record-question` | actor-stage (§6.4, judgment) | The design question the prototype must answer is recorded. |
| `20L-build-logic` | actor-stage (§6.4, judgment) | A logic prototype is built to answer the recorded question. |
| `20U-build-variants` | actor-stage (§6.4, judgment) | UI variants are built to answer the recorded question. |
| `30-hand-off` | actor-stage (§6.4, judgment) | The prototype and its answer are handed off. |
| `40-capture` | actor-stage (§6.4, judgment) | A validated decision is folded into real code and rewritten to production standards; the throwaway is preserved on a throwaway branch. |

## Notes for reviewers

The A/U branch at `20L`/`20U` is the corpus's cleanest evidence for *conditional* procedure. It is representable today as one selection stage (`00-select-branch`) plus mutually-exclusive downstream stages — recorded as grammar pressure for a future conditional-stage schema extension, not an engine gap (the current linear `workflow.toml` requires both stage directories to exist; the non-selected one is a documented no-op for that run).

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
