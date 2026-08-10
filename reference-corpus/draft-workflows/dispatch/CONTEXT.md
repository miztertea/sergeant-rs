# Dispatch
Draft workflow package — candidate **W8** `dispatch` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Given a project, a brief or tracked task, and a repository set, produce one durable task with an isolated work surface, a rendered mission brief, and a running agent per repository — with every side effect validated and gated before the next repository's dispatch begins.

## Trigger

Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-check-queue-and-plan` | actor-stage (§6.4, judgment) | Either an existing tracked task supplies brief/branch/context, or a free-form brief plus explicit repo list is confirmed as accurate before anything is created. |
| `05-classify-risk` | actor-stage (§6.4, judgment) | The objective is routed to the standard-isolated path or forced onto an explicit intent-file path by a fixed safety-sensitive keyword set. |
| `10-preflight-capabilities` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Harness, model tuple, identity and pane/session bindings are all validated and rejected before any durable state exists. |
| `15-check-admission` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The fleet-wide admission lock is held only across the first side effect, then released. |
| `20-prepare-intent` | actor-stage (§6.4, judgment) | One canonical intent revision exists and is written identically to fleet state and every selected work surface. |
| `30-create-tracked-work` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | All-or-nothing task creation across every target repo, rolled back on any failure. |
| `40-reconcile-before-launch` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Bulk fleet reconciliation runs before new work is created. |
| `50-acquire-surface` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | An isolated work surface per repo at a deterministic location; a branch already carrying unpushed committed work is refused unless explicitly adopted. |
| `60-render-brief` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Mission, merged instructions, dependency notes, delivery requirements and any verbatim user override are durably carried to the worker before it starts. |
| `70-launch-and-record` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Launch evidence is written `intended` then promoted to `confirmed` only on observed readiness; every per-repo failure records an orphaned status with a diagnostic before the loop aborts. |
| `80-monitor` | actor-stage (§6.4, judgment) | Escalations are read in full, human decisions obtained without inference, delivered to the exact task/repo pair. |
| `90-reconcile-fleet` | actor-stage (§6.4, judgment) | Per-repo verification of pinned scope, validation, review artifacts, zero blocking findings, CI, threads, and dependency merge order — never complete merely because PRs exist. |

## Relationships to other workflows

- `15-check-admission` delegates to **drain-fleet**.
- `80-monitor` delegates to **respond-to-worker**.

## Notes for reviewers

**Obsolete-mechanism stress test (§8.2).** The `dispatch` skill's tmux/sentinel/worker-Bash machinery (pane identity, pane-as-notification-channel, pane-as-liveness-signal, the nudge loop) carried none of the stage boundaries above — see `reference-corpus/synthesis.md` §4 clusters M1–M4 for the mechanism-vs-policy separation. What survived: preflight-before-side-effect, all-or-nothing tracked-work creation, one canonical intent revision, durable brief delivery, intended→confirmed launch evidence, per-repo failure recorded rather than silent. Worker-contract content this workflow *authors* but does not itself execute (BU-P5-075/076/078/079/080/081/082/083/084/085/086/089) is the input to `worker-mission` and `route-review-findings`.

Reviewers flagged this as the corpus's largest single cluster (63 units, 12 stages) — see `reference-corpus/synthesis.md` §8 note 1: either it is genuinely one procedure with twelve checkpoints, or it should split at `70-launch-and-record` into a plan-and-validate workflow and a launch-fleet workflow. Recorded as an open question for the classification ledger, not resolved here.

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P1-057`, `BU-P6-123`, `BU-P7-073`, `BU-P7-078`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
