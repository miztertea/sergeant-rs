# Monitor Fleet
Draft workflow package — candidate **W13** `monitor-fleet` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Observe fleet state without mutating it.

## Trigger

An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-snapshot` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A bounded, constant-size, versioned, strictly read-only snapshot; `busy:true` only with a verified witness, otherwise `busy:null`. |
| `10-evaluate-liveness` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Identity plus recent meaningful progress with a defined fallback chain; a stalled live worker records a non-terminal diagnostic, never an automatic kill. |
| `20-reconcile-terminal` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A `done` status with an empty result is refused as completion and marked orphaned; terminal recycling is identity-bound and settles the lease first. |
| `30-background-watch` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Idempotent start, failed-start detection, stale-unit cleanup, graceful on unsupported platforms. |

## Notes for reviewers

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P6-101`, `BU-P6-104`, `BU-P6-105`, `BU-P7-100`, `BU-P8-072`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
