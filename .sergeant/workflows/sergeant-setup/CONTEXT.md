# Sergeant Setup
Draft workflow package — candidate **W3** `sergeant-setup` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Capture a complete project definition through interview, and track any
capability gap discovered along the way as approved work — the judgment
portion of "bring an installation from any partial state to a
verified-complete state" that `sgt init`/`sgt doctor` don't already cover
mechanically (see "Retired" note under Stages below).

## Trigger

First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `05-file-capability-gaps` | actor-stage (§6.4, judgment) | Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap. |
| `30-project-interview` | actor-stage (§6.4, judgment) | A complete project definition is captured from the user, previewed in full, and written only after confirmation. |

**Retired (MVP-5 F2 execution-surface re-triage, 2026-08-12):**
`00-detect-prerequisites`, `10-install-commands`, `20-global-config`,
`40-repair-existing`, `60-task-tracking-init`, `70-optional-capabilities` —
the SPLIT verdict (`docs/icm/retriage-2026-08-11.md`) found this bootstrap/
repair machinery collides with shipped `sgt init`/`sgt doctor` (§2a's "who
drives" test: mechanical detection/repair reading `sgt` state, not
alternative-weighing). Their content moved to
`docs/icm/re-homing-record-2026-08-12.md`, not restated here; provenance
for the retired stages is preserved in this repository's git history. The
two surviving stages above are this package's SPLIT-verdict "workflow
core" — see each one's own CONTEXT.md for how it now stands without the
retired stages' upstream artifacts.

## Standing constraints (Layer 3, `_config/standing-constraints.md`)

Write only to Sergeant-owned paths. Never write to other tools' config surfaces. Never auto-initialize external tools without explicit consent. These apply across every stage of this workflow, not to one checkpoint — they belong in this workflow's `_config/` (Layer 3), not in any single stage's Inputs table.

## Notes for reviewers

`sergeant-install` (P8, from `docs/getting-started.md`) documents the identical procedure as a checklist rather than as phases and is merged into this workflow (conflict X10). The two partitions' prerequisite lists differ (P5 omits Python 3 and Node; P8 adds them) — that difference is unresolved and is preserved as an open item in provenance.md rather than silently picking one list.

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P5-010`, `BU-P8-041`, `BU-P8-051`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not. Per A11, this single statement replaces the per-stage "read pane as..." reader-note blocks that were previously copy-pasted into `00-detect-prerequisites` and `90-completion-summary`'s (now folded into `70-optional-capabilities`) `CONTEXT.md` files.

**N1 adjudication A4 (finding N1-BH-02).** `50-sync-and-verify` and `90-completion-summary` carried no argument beyond the §6.5 boilerplate and demote by default, folding into `60-task-tracking-init` and `70-optional-capabilities` respectively as helper invocations. `00-detect-prerequisites` carried an Additional note, but that note was about a citation cross-reference, not a checkpoint-differentiation argument; judged independently against §6.3's reimplementation test, the stage's own risk-acceptance gate (`BU-P5-013`) is structurally identical to this package's other confirm-gated actor stages and is **kept**, reclassified from §6.5 to §6.4. See `provenance.md`'s "Adjudication A4" section for the full reasoning on all three.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
