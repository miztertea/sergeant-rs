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
| `20-merge-or-fail` | stage (§6.3, deterministic-machinery candidate; kept per A4 — see stage CONTEXT.md; absorbs `10-extract-per-repo` and `30-publish-atomically`) | All-or-nothing: any repo's extraction failure fails the run before merge. |

## Notes for reviewers

`40-consume` (BU-P5-105/106/112: query the published graph for focused questions or read the report for broad context) failed the §6.3 reimplementation test — "I ran a query" is not a checkpoint operators track — and is demoted to shared context/helper (see `reference-corpus/synthesis.md` §3 for the destination map; this milestone does not materialize helper-map.md).

**N1 adjudication A4 (finding N1-BH-02).** `10-extract-per-repo` and `30-publish-atomically` carried no argument beyond the §6.5 boilerplate and demote by default. `20-merge-or-fail` carried a real Additional note ("we never publish a partial graph" outlives any particular merger implementation) that passes §6.3's reimplementation test — the all-or-nothing guarantee *is* the checkpoint, not an artifact of implementation — and is **kept**. Both demoted stages fold into `20-merge-or-fail` as helper invocations. The behavior units survive — see that stage's "Helpers (folded per N1 adjudication A4)" section and `provenance.md`.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
