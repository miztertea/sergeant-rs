# Resolving Merge Conflicts
Draft workflow package — candidate **W26** `resolving-merge-conflicts` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Resolve an in-progress git merge/rebase conflict without inventing behavior or aborting.

## Trigger

A git merge or rebase is in a conflicted state.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-assess-state` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The current merge/rebase state is assessed. |
| `10-research-intent` | actor-stage (§6.4, judgment) | The intent behind each conflicting side is researched. |
| `20-resolve-hunks` | actor-stage (§6.4, judgment) | Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted. |
| `30-validate` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Typecheck, tests, format run in that order. |
| `40-finish` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The merge/rebase is completed. |

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
