# Worker Mission (software-change)
Draft workflow package — candidate **W9** `worker-mission` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

From a rendered brief, produce a merged-ready change with evidence — the contract a dispatched worker delivers against.

## Trigger

A worker starts against a rendered brief.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-pin-scope` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Refs fetched, a fixed base commit pinned, base SHA/commit list/diff scope recorded before implementation. |
| `10-triage-and-route` | actor-stage (§6.4, judgment) | Full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure. |
| `20-implement` | actor-stage (§6.4, judgment) | The discipline chosen at `10-triage-and-route` runs to its own completion. |
| `30-independent-review` | actor-stage (§6.4, judgment) | Every axis named in the brief's authoritative list runs as a separate, non-contaminating parallel review; outputs unblended. |
| `40-escalate-or-continue` | actor-stage (§6.4, judgment) | A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete. |
| `50-publish-result` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Handoff evidence recorded from the verified work surface; readiness bounded and reported rather than hanging. |

## Relationships to other workflows

- `20-implement` delegates to **diagnose-bug, prototype, tdd, implement, or deepen-module (whichever 10-triage-and-route selected)**.

## Notes for reviewers

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P7-110`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
