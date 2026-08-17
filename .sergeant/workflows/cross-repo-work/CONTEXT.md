# Cross-Repo Work
Draft workflow package — candidate **W7** `cross-repo-work` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from frozen upstream evidence per
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
- `60-reconcile` *names* (does not invoke) `dispatch`'s fleet reconciliation
  as an adjacent, owned procedure (N1 adjudication A8, BH-10). It does
  **not** name a `reconcile-and-cleanup-fleet` package — no such package
  exists (`docs/icm/re-homing-record-2026-08-12.md` line 25): its per-repo
  teardown half was absorbed into `recovery.rs`'s automatic reconciliation,
  and its multi-repo fleet-grouping/cleanup half is doctrinally unbuilt
  (currently ruled out by the North Star's "fleet as a domain object"
  line). The underlying wish for a real composed cleanup procedure is
  recorded as evidence for existing engine-gap **G6**
  (`reference-corpus/engine-pressure.md`), not as a currently owned
  procedure this stage's output feeds.

## Authority envelope

This workflow receives an already-defined multi-repository objective; it
does not itself decide whether work should exist, only how to decompose an
already-scoped cross-repository objective into an ownership/dependency/
delivery plan.

### Workflow may decide
- Which repository owns each required behavior, resolved from the project
  graph and existing contracts first (`10-assign-ownership`).
- Which evidence justifies a dependency edge, and how to break a
  genuinely coupled cycle (`20-define-dependency-order`).
- Each repository's concrete delivery-gate content from inspected state
  and the dependency graph (`40-define-delivery-gates`).
- Whether the user requested planning-only or implementation
  (`50-handoff-or-stop`).
- Which reconciled fact (PR/CI/thread/merge-order/deploy state) applies to
  each named repository (`60-reconcile`).

### Workflow may not decide
- Ask the user about every ownership ambiguity rather than only genuinely
  contested cross-repository ownership.
- Let a cyclic dependency graph reach dispatch.
- Stash, reset, switch, or clean repository state during planning.
- Edit several repositories itself, or perform the dispatched worker's own
  git mutations (`git checkout -b`, `git push -u origin`, `gh pr create`)
  inline.
- Report the cross-repo outcome complete without a terminal result or an
  explicit preserved blocker for every named repository, or assert
  anything about repos, tasks, or fleet state outside its own plan's repo
  set.

### Human or Captain gates
- Genuinely contested cross-repository ownership (`10-assign-ownership`).
- A required data/security/destructive decision for a repository's gate
  that is still missing or unresolved (`40-define-delivery-gates`).

### Decision record
Material decisions (ownership assignments, dependency edges, delivery
gates, the planning-only/implementation determination, reconciled facts)
are recorded in each stage's own turn and surfaced through `needs_input`
where applicable; this workflow declares no separate decision-log file.

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

**Correction (ICM-R3, 2026-08-16):** the A8 note previously also named
`reconcile-and-cleanup-fleet` as an adjacent owned procedure for the
multi-repo cleanup half. No such package exists — see "Relationships to
other workflows" above for the corrected characterization.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
