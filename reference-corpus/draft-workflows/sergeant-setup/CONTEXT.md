# Sergeant Setup
Draft workflow package — candidate **W3** `sergeant-setup` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Bring an installation from any partial state to a verified-complete state without ever silently reconfiguring anything the operator did not consent to.

## Trigger

First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-detect-prerequisites` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Every checked tool is classified present / installable / unsupported; required gaps stop the run unless the user accepts the risk. |
| `05-file-capability-gaps` | actor-stage (§6.4, judgment) | Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap. |
| `10-install-commands` | actor-stage (§6.4, judgment) | Commands resolve on PATH, verified; failure stops with the expected source paths named. |
| `20-global-config` | actor-stage (§6.4, judgment) | One machine-wide `dev_root` exists and parses; an existing file is never overwritten without backup + diff + confirmation. |
| `30-project-interview` | actor-stage (§6.4, judgment) | A complete project definition is captured from the user, previewed in full, and written only after confirmation. |
| `40-repair-existing` | actor-stage (§6.4, judgment) | An existing definition is validated, a minimal diff shown, and changes applied only after confirmation with a mandatory post-confirmation backup. |
| `50-sync-and-verify` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The four verification commands run in fixed order, stopping at the first failure. |
| `60-task-tracking-init` | actor-stage (§6.4, judgment) | Tracked-work storage initialized per registered repo, each behind explicit consent. |
| `70-optional-capabilities` | actor-stage (§6.4, judgment) | Worktree pools and graph output initialized only where explicitly desired; declining never marks setup incomplete. |
| `90-completion-summary` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Every checklist item resolved as `[ok]` / `[skipped]` / `[issue: id]`. |

## Standing constraints (Layer 3, `_config/standing-constraints.md`)

Write only to Sergeant-owned paths. Never write to other tools' config surfaces. Never auto-initialize external tools without explicit consent. These apply across every stage of this workflow, not to one checkpoint — they belong in this workflow's `_config/` (Layer 3), not in any single stage's Inputs table.

## Notes for reviewers

`sergeant-install` (P8, from `docs/getting-started.md`) documents the identical procedure as a checklist rather than as phases and is merged into this workflow (conflict X10). The two partitions' prerequisite lists differ (P5 omits Python 3 and Node; P8 adds them) — that difference is unresolved and is preserved as an open item in provenance.md rather than silently picking one list.

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P5-010`, `BU-P8-041`, `BU-P8-051`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
