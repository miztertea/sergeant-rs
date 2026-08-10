# Project Graph
Draft workflow package — candidate **W2** `project-graph` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Produce exactly one merged, published graph per project, outside every source repository, usable for architecture questions.

## Trigger

Architecture work needs whole-project structure, or the operator asks for a graph/refresh.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-resolve-output-path` | actor-stage (§6.4, judgment) | One project-level output path is confirmed (or requested from the user) and is outside every source repo. |
| `10-extract-per-repo` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Per-repo extraction completed, with in-source output staged out of the way and code-only fallback when no LLM key exists. |
| `20-merge-or-fail` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | All-or-nothing: any repo's extraction failure fails the run before merge. |
| `30-publish-atomically` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Readers see the complete old or complete new graph, never a torn state; a failed swap leaves the previous output valid. |

## Notes for reviewers

`40-consume` (BU-P5-105/106/112: query the published graph for focused questions or read the report for broad context) failed the §6.3 reimplementation test — "I ran a query" is not a checkpoint operators track — and is demoted to shared context/helper (see `reference-corpus/synthesis.md` §3 for the destination map; this milestone does not materialize helper-map.md).

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
