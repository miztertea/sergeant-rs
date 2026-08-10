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
| `02-check-queue` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A matching tracked task is reused, or one is created because none is canonical. |
| `03-choose-mode` | actor-stage (§6.4, judgment) | Direct or dispatch is selected on the four stated criteria. |
| `04-reconcile-state` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Active workers, branches, worktrees, retained gates and handoffs are inspected; preserved work is resumed rather than duplicated. |
| `05-confirm-decisions` | actor-stage (§6.4, judgment) | Only genuinely unresolved scope/risk decisions are put to the user. |
| `06-execute` | actor-stage (§6.4, judgment) | Control passes to `direct-implementation` or `dispatch`. |
| `07-monitor` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Progress is evidenced by recent meaningful events plus exact process identity. |
| `08-handle-decisions` | actor-stage (§6.4, judgment) | Each gate resolved with a recorded human decision where required. |
| `09-reconcile-deliver` | actor-stage (§6.4, judgment) | PRs, merge order, merges/deployments and cleanup eligibility are settled. |

## Relationships to other workflows

- `01-load-context` delegates to **load-project**.
- `06-execute` delegates to **direct-implementation or dispatch (chosen at 03-choose-mode)**.

## Notes for reviewers

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P1-032`, `BU-P1-038`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
