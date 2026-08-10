# Recover Stalled Worker
Draft workflow package — candidate **W11** `recover-stalled-worker` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

One bounded recovery attempt for a stalled worker: converge on a replacement or escalate — never guess.

## Trigger

A worker is `in_progress` with a stall classification recorded by the watcher.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-collect-signals` | actor-stage (§6.4, judgment) | Four signals are collected together before any kill/relaunch decision. |
| `10-preflight` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Stall proof, lease convergence, drain check, relaunch-metadata completeness, and old identity all run to completion before the attempt is stamped. |
| `20-launch-replacement` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The replacement is validated live before the original is retired. |
| `30-retire-original` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The original is retired only after the replacement is proven live. |
| `40-escalate-on-second-attempt` | actor-stage (§6.4, judgment) | Exactly one bounded recovery attempt is made; a second stall escalates to needs-input. |
| `50-escalate-undocumented` | actor-stage (§6.4, judgment) | An undocumented/unrecognized stall class escalates rather than being guessed at. |

## Notes for reviewers

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P6-072`, `BU-P6-073`, `BU-P6-075`, `BU-P7-093`, `BU-P7-094`, `BU-P7-095`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
