# Respond to Worker
Draft workflow package — candidate **W10** `respond-to-worker` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

A blocked/needs-input/waiting/orphaned worker is durably given exactly one decision, applies it exactly once, and returns to forward progress.

## Trigger

A worker has published an escalation and a human decision exists.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-precondition-check` | actor-stage (§6.4, judgment) | Exact question read, only genuinely missing decisions asked, decision recorded in tracked work, no unconsumed generation already pending. |
| `10-validate-target` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The target's status is one of the four respondable states and its recorded identity/ownership evidence verifies; anything else refuses. |
| `20-publish-response` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The response is durably stored (even under an active drain) before any delivery is attempted. |
| `30-deliver-and-accept` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Bounded readiness gate; on timeout, a nonce-scoped unreachable record plus a recoverable gate — never a fabricated acknowledgement. |
| `40-apply-and-acknowledge` | actor-stage (§6.4, judgment) | Decision applied once, truthful status restored, applied id/generation/status recorded, then acknowledged from the owning context. |
| `50-archive-evidence` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Body, generation, applied status and proof archived atomically; the recorded generation is fixed at acknowledgement time. |
| `60-notify-coordinator` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The update is classified into exactly one durable event kind and recorded; live transports are optional on top. |
| `70-relaunch-if-needed` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Convergence attempted through the single finalizer before any refusal; superseded identities preserved as evidence. |

## Notes for reviewers

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P6-080`, `BU-P7-041`, `BU-P7-048`, `BU-P7-058`, `BU-P7-059`, `BU-P7-060`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
