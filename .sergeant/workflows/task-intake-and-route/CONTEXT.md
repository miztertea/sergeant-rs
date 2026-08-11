# Task Intake and Route
Draft workflow package — candidate **W5** `task-intake-and-route` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

The standing entry procedure every task passes through before any implementation workflow starts: it turns a user request into a chosen, scoped execution mode.

## Trigger

Any task the user brings.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `01-load-context` | actor-stage (§6.4, judgment) | Owning repositories, inherited instructions and cross-repo dependencies are known. |
| `03-choose-mode` | actor-stage (§6.4, judgment; absorbs `02-check-queue` per A4) | Direct or dispatch is selected on the four stated criteria. |
| `05-confirm-decisions` | actor-stage (§6.4, judgment; absorbs `04-reconcile-state` per A4) | Only genuinely unresolved scope/risk decisions are put to the user. |
| `06-execute` | actor-stage (§6.4, judgment) | Control passes to `direct-implementation` or `dispatch`. |
| `08-handle-decisions` | actor-stage (§6.4, judgment; absorbs `07-monitor` per A4) | Each gate resolved with a recorded human decision where required. |
| `09-reconcile-deliver` | actor-stage (§6.4, judgment) | PRs, merge order, merges/deployments and cleanup eligibility are settled. |

## Relationships to other workflows

- `01-load-context` delegates to **load-project**.
- `06-execute` delegates to **direct-implementation or dispatch (chosen at 03-choose-mode)**.

## Notes for reviewers

**N1 adjudication A4 (finding N1-BH-02).** This package originally decomposed the standing entry procedure into nine stages mirroring AGENTS.md's nine numbered steps. `02-check-queue`, `04-reconcile-state`, and `07-monitor` carried no argument beyond the §6.5 deterministic-machinery boilerplate, so all three demote by default and fold forward as helper invocations into the judgment stage each one directly precedes (`03-choose-mode`, `05-confirm-decisions`, `08-handle-decisions` respectively). The behavior units survive — see each surviving stage's "Helpers (folded per N1 adjudication A4)" section and `provenance.md`.

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P1-032`, `BU-P1-038`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
