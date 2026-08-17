# 60-reconcile: reconcile the repo set's completion facts

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-handoff-or-stop/output/README.md | L4 | upstream artifact produced by `50-handoff-or-stop` |

## Purpose

PR URLs, heads, CI, review threads, merge and deployment order, and terminal task/fleet state — **scoped to the specific repo set this Work's plan named** (N1 adjudication A8, BH-10). `dispatch` owns fleet-wide reconciliation (its automatic pre-launch sweep, folded under A4 into `dispatch/80-monitor`); this stage does not re-derive or invoke that procedure. It is an adjacent, owned procedure this stage names for the reader, not one it composes with — no `@@name` reference or delegation substitutes for a real child-workflow invocation, because none exists yet at this milestone (`docs/icm/convention.md` §4 rule 1; the underlying pressure to invoke it for real is recorded as evidence for engine-gap G6 in `sergeant-rs-workspace/knowledge/evidence/reference-corpus/engine-pressure.md`, not as a new claim). This stage's terminal task/fleet-state reporting does not name a `reconcile-and-cleanup-fleet` package as its consumer — no such package exists (see the workflow-level `CONTEXT.md`'s "Relationships to other workflows").

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

For the repositories named in this Work's plan only: PR URLs, heads, CI, review threads, merge and deployment order, and terminal task/fleet state for those repos. This stage does not assert or reconcile anything about repos, tasks, or fleet state outside its own plan's repo set — that is `dispatch`'s own adjacent, owned fleet-wide reconciliation territory (see "Scope note" below for the multi-repo cleanup half, which no live package currently owns).

## Behavior contract

- **After dispatched workers finish, cross-repo-work reconciles PR URLs and final heads, required CI and unresolved review threads, merge order from dependency edges, deployment order and cross-repo release notes, and terminal td/fleet state and cleanup eligibility.**
  (trigger: dispatch has completed for the plan's repositories; outcome: the multi-repo outcome is reconciled against every planned gate, not just individual PR existence)
- **cross-repo-work never reports the cross-repo outcome complete until every owning repository has a terminal result or an explicit preserved blocker.**
  (trigger: reconciliation is being evaluated for completeness; outcome: partial completion is never reported as done)

**Scope note (N1 adjudication A8, BH-10; corrected ICM-R3 2026-08-16):** the "terminal td/fleet state and cleanup eligibility" phrase in the behavior contract above is read here as reporting terminal state and cleanup eligibility *for the repos this plan named*, not as this stage owning fleet-wide reconciliation or cleanup. `dispatch` owns the automatic pre-launch fleet reconciliation sweep. No live package owns the actual multi-repo cleanup decision and mutation once every repo is terminal — a `reconcile-and-cleanup-fleet` package was never built (per-repo teardown was absorbed into `recovery.rs`'s automatic reconciliation; the multi-repo fleet-grouping/cleanup half is doctrinally foreclosed by the North Star's "fleet as a domain object" ruling, `docs/icm/re-homing-record-2026-08-12.md` line 25). This stage only reports the repo-set-specific completion facts; the underlying wish for a real composed cleanup procedure is recorded as evidence for engine-gap G6, not as a currently owned procedure.

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- Never report the cross-repo outcome complete without a terminal result or an explicit preserved blocker for every named repository.
- Never assert or reconcile anything about repos, tasks, or fleet state outside this plan's own repo set (A8 scope note).

### J2 — delegated to this stage
- Which reconciled fact (PR/CI/thread/merge-order/deploy state) applies to each named repository.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once every owning repository named in this plan has a reconciled terminal result or an explicit preserved blocker.

### Decision evidence
The reconciled per-repo facts are this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
