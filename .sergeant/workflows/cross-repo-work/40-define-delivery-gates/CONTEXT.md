# 40-define-delivery-gates: inspect repository state, then define delivery gates

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-define-dependency-order/output/README.md | L4 | upstream artifact produced by `20-define-dependency-order` |

## Purpose

Per-repo gate: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions. N1 adjudication A4 folded `30-inspect-repository-state` in ahead of this stage's own judgment: swapping the status-inspection implementation would leave this stage's checkpoint (a complete, checkable per-repo gate) unchanged, so the read-only inspection is a helper this stage runs first, not an independent checkpoint.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

Non-main branches, uncommitted changes, ahead/behind state, active worktrees, and preserved workers are recorded for every owning repository without mutating anything; then every per-repo delivery gate is defined: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions.

## Behavior contract

- **Every per-repository delivery gate must include: the owning td task (or its creation requirement), the fixed point and preserved source state, repository-specific test/lint/typecheck/build commands, Standards and Spec review sources, PR dependency and deployment order, and any already-approved or still-missing data/security/destructive decisions.**
  (trigger: delivery gates are being defined per repository; outcome: every repository's brief has a complete, checkable gate set before dispatch)
- **The cross-repo plan is complete only when every owning repository has one implementation brief, acceptance evidence, and an acyclic dependency position.**
  (trigger: delivery gates have been drafted for every repository; outcome: the plan's completion condition is explicit and checkable)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- Repository state inspection is strictly read-only: never stash, reset, switch, or clean.
- The plan's completion condition — brief + acceptance evidence + acyclic position for every owner — is fixed, not a judgment call.

### J2 — delegated to this stage
- Defining each repository's concrete delivery-gate content from the inspected state and dependency graph.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- Repository state conflicts with the requested outcome: stop for a decision rather than routing it into the worker brief.
- A required data/security/destructive decision for a repository's gate is still missing or unresolved.

### Completion boundary
This stage may complete only once every owning repository has one implementation brief, acceptance evidence, and an acyclic dependency position — or the stage has stopped at one of the J0 cases above.

### Decision evidence
The per-repository delivery gate is this stage's own durable output, recorded per `output/README.md`.

## Helper invocations (folded stages, N1 adjudication A4)

**1. inspect repository state** (formerly `30-inspect-repository-state`) — non-main branches, uncommitted changes, ahead/behind, worktrees, preserved workers recorded without mutating anything. Classified at extraction as deterministic machinery (§6.5) with no "Additional note" arguing otherwise; swapping the status-inspection implementation leaves the checkpoint (a read-only record of repo state) unchanged, so it folds in here as a helper invocation run before gate definition:

- **cross-repo-work runs the status command and records non-main branches, uncommitted changes, ahead/behind state, active worktrees, and preserved workers for every owning repository before planning proceeds.**
  (trigger: ownership and dependencies are being established; outcome: the plan accounts for each owning repository's actual current state)
- **cross-repo-work never stashes, resets, switches, or cleans repository state during planning; it either routes an existing canonical branch/worktree into the worker brief or stops for a decision when state conflicts with the requested outcome.**
  (trigger: planning inspects a repository with pre-existing state; outcome: planning is strictly read-only with respect to repository state)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
