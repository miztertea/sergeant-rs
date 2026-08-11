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
| `10-research-intent` | actor-stage (§6.4, judgment) | The intent behind each conflicting side is researched; folds the demoted `00-assess-state` checkpoint as a helper (N1 adjudication A4). |
| `20-resolve-hunks` | actor-stage (§6.4, judgment) | Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted; folds the demoted `30-validate` and `40-finish` checkpoints as helpers (N1 adjudication A4). |

`00-assess-state`, `30-validate`, and `40-finish` were demoted per N1 adjudication A4 (finding N1-BH-02): each was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate. Their behavior units survive, folded into the adjacent judgment-bearing stage as helper invocations — see each stage's own `CONTEXT.md` and `provenance.md`'s "Adjudication A4" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
