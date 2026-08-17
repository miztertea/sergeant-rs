# Dispatch
Draft workflow package — candidate **W8** `dispatch` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

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

## Authority envelope

This workflow receives an already-admitted Work intent (a project, a brief or tracked task, and a repository set).

### Workflow may decide
- Whether an existing td task or a free-form brief supplies the plan (`00-check-queue-and-plan`).
- Which itemized gates a repo has met at reconciliation time (`90-reconcile-fleet`).

### Workflow may not decide
- Routing a safety-sensitive objective onto the standard-isolated path — the keyword set is fixed (`05-classify-risk`).
- Widening the admission lock's hold window beyond the first durable side effect (`15-check-admission`).
- Resolving a worker's escalation without an explicit, uninferred human decision (`80-monitor`).

### Human or Captain gates
- Confirming the stated dispatch plan before anything is created.
- Every worker escalation.

### Decision record
Material decisions are recorded per-stage in each stage's own output artifact.

## No `sgt dispatch` verb (skew-check-2026-08-17 finding 3)

Every stage in this package phrases behavior as "sgt-dispatch does X" / "sgt-dispatch must Y," naming the upstream bash tool this package decomposes. **`sgt` has no `dispatch` verb, hyphenated or otherwise** — confirmed by `sgt --help` (top-level verb list: `daemon`, `status`, `run`, `work`, `respond`, `retry`, `extend`, `cancel`, `watch`, `analytics`, `tui`, `doctor`, `init`, `repo`, `group`, `claude`, `codex`, `opencode`, `goose` — no `dispatch`, no `harness`) and by running `sgt dispatch --help` directly (`error: unrecognized subcommand 'dispatch'`). Wherever a stage says "sgt-dispatch," read it as upstream-tool provenance for the behavior unit, never as a present-tense `sgt` CLI invocation.

The one thing `sgt-dispatch` names that this package genuinely needs a concrete mapping for — running this workflow at all — is `sgt run --workflow dispatch`. That mapping is mechanical and works today (see finding 3's own caveat about a fresh estate 422ing on any non-default workflow name, engine gap tracked separately as skew-check finding 6 / issue #165 — not this package's concern to fix).

## Relationships to other workflows

**Corrected 2026-08-16, ICM-R3:** neither of the two packages named below exists in this repository — both are open, unbuilt engine gaps, not live delegations.

- `15-check-admission` holds and releases the fleet-wide admission lock itself, across exactly one durable side effect — it does not delegate to a `drain-fleet` workflow (unbuilt, engine-gap G4).
- `80-monitor` delivers escalation responses via the shipped `sgt respond` command / `POST /v1/work/{id}/input` — it does not delegate to a `respond-to-worker` workflow (unbuilt).

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
`15-check-admission` (its own "Additional note" conceded that the
checkpoint — nothing was created if the preflight failed — is unchanged by
swapping which probes implement the check, so it does *not* survive §6.3's
reimplementation test in the stage's favor — exactly what marks it a
helper); `30-create-tracked-work`,
`40-reconcile-before-launch`, `50-acquire-surface`, `60-render-brief`, and
`70-launch-and-record` folded into `80-monitor`. `15-check-admission`
itself was judged case-by-case and **kept**: its "Additional note" argues a
real cross-workflow dependency (this stage's outcome is produced by running
**drain-fleet** to completion, not by swapping a local implementation
detail), which does not reduce under §6.3's test. Stage count dropped from
12 to 6; no behavior unit was deleted — see `provenance.md`'s "Adjudication
A4" section and each surviving stage's "Helper invocations" section for the
full disposition.

**Obsolete-mechanism stress test (§8.2).** The `dispatch` skill's tmux/sentinel/worker-Bash machinery (pane identity, pane-as-notification-channel, pane-as-liveness-signal, the nudge loop) carried none of the stage boundaries above — see `sergeant-rs-workspace/knowledge/evidence/reference-corpus/synthesis.md` §4 clusters M1–M4 for the mechanism-vs-policy separation. What survived: preflight-before-side-effect, all-or-nothing tracked-work creation, one canonical intent revision, durable brief delivery, intended→confirmed launch evidence, per-repo failure recorded rather than silent. Worker-contract content this workflow *authors* but does not itself execute (routed here at N1 verifier round 2 finding V3) is the input to `worker-mission` and `route-review-findings`.

Reviewers originally flagged this as the corpus's largest single cluster (63 units, 12 stages) — see `sergeant-rs-workspace/knowledge/evidence/reference-corpus/synthesis.md` §8 note 1: either it is genuinely one procedure with twelve checkpoints, or it should split at `70-launch-and-record` into a plan-and-validate workflow and a launch-fleet workflow. A4's de-staging sweep (above) addresses the size concern from a different angle than a workflow split — the checkpoint count actually requiring independent judgment turns out to be five, not twelve. Whether `80-monitor`'s post-fold breadth still argues for a plan-and-validate / launch-fleet split remains an open question for the classification ledger; not resolved here.

## Provenance

See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/dispatch.md` for the complete stage-to-behavior-unit mapping and workflow-level citations. (ICM-R3 correction: the prior text pointed at a workflow-local `provenance.md` that does not exist under `.sergeant/workflows/dispatch/`.)
