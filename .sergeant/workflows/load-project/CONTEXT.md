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

**SPLIT verdict executed (MVP-5 F2, 2026-08-12).** The execution-surface
re-triage (`docs/icm/retriage-2026-08-11.md`) confirmed those same four
functions as CLI-SURFACE and retired their content from
`20-register-or-edit`'s folded helpers — see that stage's own CONTEXT.md
and `docs/icm/re-homing-record-2026-08-12.md`. This workflow's three stages
(00/10/20) stay, unchanged in their own judgment content, as the SPLIT
verdict's surviving "workflow core"; they still describe upstream's
`~/.config/sergeant/<project>.yaml` registry mechanism, which has no
sergeant-rs analog yet (sergeant-rs's estate model is `sergeant.toml`,
per-directory, not a multi-project registry) — translating this package to
that model is out of this re-homing pass's scope.

**N1 adjudication A4 (finding N1-BH-02).** This package originally ended in two further stages, `30-sync-repositories` and `40-report-state`. `30-sync-repositories` carried no argument beyond the §6.5 boilerplate and demotes by default. `40-report-state` carried an Additional note that was weighed against §6.3's reimplementation test and failed (it is, in its own words, "closer to a query than a checkpoint"). Both originally folded into `20-register-or-edit` as helper invocations covering `list-projects`/`project-status`/`project-sync`/`project-task-list`.

**Superseded by the SPLIT verdict above.** The MVP-5 F2 execution-surface
re-triage retired those same four functions to CLI-SURFACE instead
(`docs/icm/re-homing-record-2026-08-12.md`) — `20-register-or-edit` no
longer carries a "Helpers (folded per N1 adjudication A4)" section; see
its own "Retired helper content" section for what replaced it and where
those behavior units actually live now.

## Provenance

`provenance.md` (referenced above and historically in this package's own
stage files) was never actually created for `load-project` — the real
stage-to-behavior-unit trail is
`docs/gauntlet/promoted-provenance/load-project.md`.
