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
| `15-check-admission` | actor-stage (§6.4, judgment — real cross-workflow delegation; see Adjudication notes) | Preflight capabilities are validated (folded helper) and the fleet-wide admission lock is held only across the first side effect, then released. |
| `20-prepare-intent` | actor-stage (§6.4, judgment) | One canonical intent revision exists and is written identically to fleet state and every selected work surface. |
| `80-monitor` | actor-stage (§6.4, judgment) | Fleet reconciliation, tracked-work creation, surface acquisition, brief rendering, and launch-and-record all run first (folded helpers, in that order); escalations are then read in full, human decisions obtained without inference, delivered to the exact task/repo pair. |
| `90-reconcile-fleet` | actor-stage (§6.4, judgment) | Per-repo verification of pinned scope, validation, review artifacts, zero blocking findings, CI, threads, and dependency merge order — never complete merely because PRs exist. |

## Relationships to other workflows

- `15-check-admission` delegates to **drain-fleet**.
- `80-monitor` delegates to **respond-to-worker**.

## Adjudication notes (A3, A4)

**A3 (BH-01, ordering).** The extraction originally sequenced
`30-create-tracked-work` ahead of `40-reconcile-before-launch`, even though
the latter's own contract says dispatch always runs fleet reconciliation
"automatically before creating new work." N1 adjudication A3 confirmed the
finding and fixed the order: reconciliation now precedes tracked-work
creation. Because A4 (below) then folded both into `80-monitor` as helper
invocations, the fix survives as invocation order within that stage's
"Helper invocations" section, not as separate stage directories.

**A4 (BH-02, de-staging sweep).** Six of this package's twelve extracted
stages carried no argument beyond the §6.5 "candidate execute-stage
workload" boilerplate and folded into their nearest surviving
judgment-bearing neighbor: `10-preflight-capabilities` folded into
`15-check-admission` (its own "Additional note" argued the checkpoint
survives implementation swap — exactly what marks it a helper under
§6.3's reimplementation test); `30-create-tracked-work`,
`40-reconcile-before-launch`, `50-acquire-surface`, `60-render-brief`, and
`70-launch-and-record` folded into `80-monitor`. `15-check-admission`
itself was judged case-by-case and **kept**: its "Additional note" argues a
real cross-workflow dependency (this stage's outcome is produced by running
**drain-fleet** to completion, not by swapping a local implementation
detail), which does not reduce under §6.3's test. Stage count dropped from
12 to 6; no behavior unit was deleted — see `provenance.md`'s "Adjudication
A4" section and each surviving stage's "Helper invocations" section for the
full disposition.

**Obsolete-mechanism stress test (§8.2).** The `dispatch` skill's tmux/sentinel/worker-Bash machinery (pane identity, pane-as-notification-channel, pane-as-liveness-signal, the nudge loop) carried none of the stage boundaries above — see `reference-corpus/synthesis.md` §4 clusters M1–M4 for the mechanism-vs-policy separation. What survived: preflight-before-side-effect, all-or-nothing tracked-work creation, one canonical intent revision, durable brief delivery, intended→confirmed launch evidence, per-repo failure recorded rather than silent. Worker-contract content this workflow *authors* but does not itself execute (BU-P5-075/076/078/079/080/081/082/083/084/085/086/089) is the input to `worker-mission` and `route-review-findings`.

Reviewers originally flagged this as the corpus's largest single cluster (63 units, 12 stages) — see `reference-corpus/synthesis.md` §8 note 1: either it is genuinely one procedure with twelve checkpoints, or it should split at `70-launch-and-record` into a plan-and-validate workflow and a launch-fleet workflow. A4's de-staging sweep (above) addresses the size concern from a different angle than a workflow split — the checkpoint count actually requiring independent judgment turns out to be five, not twelve. Whether `80-monitor`'s post-fold breadth still argues for a plan-and-validate / launch-fleet split remains an open question for the classification ledger; not resolved here.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
