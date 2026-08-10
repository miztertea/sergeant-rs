# Load Project
Draft workflow package — candidate **W1** `load-project` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Establish, before any mutation, which repositories own the requested outcome, where they are, what instructions govern them, and what state they are in.

## Trigger

A project is named, registered, edited, synced, or listed; or repository ownership is not already established.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-resolve-project-name` | actor-stage (§6.4, judgment) | An exact registered project name is bound, or the run stops asking whether to register. |
| `10-resolve-context` | actor-stage (§6.4, judgment) | Owning repos, absolute paths, clone state, roles/groups, and the layered instruction set are recorded as the governing context. |
| `20-register-or-edit` | actor-stage (§6.4, judgment; absorbs `30-sync-repositories` and `40-report-state` per A4) | A project definition is written to the Sergeant-owned config path and validated, or the prior definition is restored. |

## Notes for reviewers

`list-projects` (BU-P6-010/011), `project-status` (BU-P6-012), `project-sync` (BU-P6-013/014), and `project-task-list` (BU-P6-035) were each extracted as standalone workflows by one partition (P6) but are command surfaces, not procedures with a bounded outcome and completion condition (§6.2) — folded into this workflow's stages instead. See synthesis.md conflict X11.

**N1 adjudication A4 (finding N1-BH-02).** This package originally ended in two further stages, `30-sync-repositories` and `40-report-state`. `30-sync-repositories` carried no argument beyond the §6.5 boilerplate and demotes by default. `40-report-state` carried an Additional note that was weighed against §6.3's reimplementation test and failed (it is, in its own words, "closer to a query than a checkpoint"). Both fold into `20-register-or-edit` (now the workflow's terminal stage) as helper invocations. The behavior units survive — see that stage's "Helpers (folded per N1 adjudication A4)" section and `provenance.md`.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
