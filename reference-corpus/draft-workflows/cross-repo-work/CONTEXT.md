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
| `30-inspect-repository-state` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Non-main branches, uncommitted changes, ahead/behind, worktrees, preserved workers recorded without mutating anything. |
| `40-define-delivery-gates` | actor-stage (§6.4, judgment) | Per-repo gate: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions. |
| `50-handoff-or-stop` | actor-stage (§6.4, judgment) | Either the plan is returned (planning-only) or control passes to dispatch; the coordinator never edits several repos itself. |
| `60-reconcile` | actor-stage (§6.4, judgment) | PR URLs, heads, CI, review threads, merge and deployment order, terminal task/fleet state. |

## Relationships to other workflows

- `50-handoff-or-stop` delegates to **dispatch**.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
