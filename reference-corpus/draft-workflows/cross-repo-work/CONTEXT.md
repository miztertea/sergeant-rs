# Cross-Repo Work
Draft workflow package — candidate **W7** `cross-repo-work` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Decompose a requested outcome across repositories and define delivery order: produce a plan in which every required behavior has exactly one owning repository, an acyclic dependency position, a brief, and acceptance evidence — before any dispatch happens.

## Trigger

Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-assign-ownership` | actor-stage (§6.4, judgment) | Exactly one owning repo per behavior, with role / deliverable / acceptance recorded. |
| `20-define-dependency-order` | actor-stage (§6.4, judgment) | An acyclic edge set in prerequisite>dependent form; cycles broken by a named contract artifact. |
| `40-define-delivery-gates` | actor-stage (§6.4, judgment) | Repository state is inspected read-only (folded helper), then a per-repo gate is defined: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions. |
| `50-handoff-or-stop` | actor-stage (§6.4, judgment) | Either the plan is returned (planning-only) or control passes to dispatch; the coordinator never edits several repos itself. |
| `60-reconcile` | actor-stage (§6.4, judgment) | PR URLs, heads, CI, review threads, merge and deployment order, and terminal task/fleet state — scoped to this plan's own repo set (N1 adjudication A8). |

## Relationships to other workflows

- `50-handoff-or-stop` delegates to **dispatch**.
- `60-reconcile` *names* (does not invoke) `dispatch`'s fleet reconciliation and `reconcile-and-cleanup-fleet`'s cleanup as adjacent, owned procedures (N1 adjudication A8, BH-10).

## Adjudication notes (A4, A8)

**A4 (BH-02, de-staging sweep).** `30-inspect-repository-state` was the
package's only stage extracted at ladder §6.5 with no "Additional note"
arguing otherwise. Swapping its status-inspection implementation leaves the
checkpoint it produces — a read-only record of repo state — unchanged, so
it folded into `40-define-delivery-gates` as a preceding helper invocation.
Stage count: 6 extracted → 5 surviving. See `provenance.md`'s "Adjudication
A4" section and `40-define-delivery-gates/CONTEXT.md`'s "Helper invocations"
section.

**A8 (BH-10, duplicated reconcile checkpoint).** This package's `60-reconcile`
and `dispatch`'s fleet reconciliation were drafted as two independent
implementations of overlapping-sounding checkpoints. Adjudicated: `dispatch`
owns fleet-wide reconciliation (its automatic pre-launch sweep); this
package's `60-reconcile` narrows to the repo-set-specific completion facts
for the repos *this Work's plan* named, and names dispatch's reconciliation
as an adjacent owned procedure rather than pretending to invoke it (no
child-workflow invocation exists at this milestone —
`docs/icm/convention.md` §4 rule 1). The underlying wish to invoke it for
real is recorded as evidence for existing engine-gap claim G6 in
`reference-corpus/engine-pressure.md`, not filed as a new claim. See
`60-reconcile/CONTEXT.md`'s "Scope note" for the stage-level disposition.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
