# 90-reconcile-fleet: reconcile fleet

## Inputs

| File | Layer | Why |
|---|---|---|
| ../80-monitor/output/README.md | L4 | upstream artifact produced by `80-monitor` |

## Purpose

Per-repo verification of pinned scope, validation, review artifacts, zero blocking findings, CI, threads, and dependency merge order — never complete merely because PRs exist.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Per-repo verification of pinned scope, validation, review artifacts, zero blocking findings, CI, threads, and dependency merge order — never complete merely because PRs exist.

## Behavior contract

- **Reconciliation after all workers finish requires verifying, per repository: pinned-base scope, focused/full validation, separate standards/spec review artifacts, an accessibility review artifact for UI-facing work, zero blocking findings, required CI, and resolved non-outdated review threads; and checking dependency order so infra merges before API before app when there is a runtime dependency.**
  (trigger: all dispatched workers report done; outcome: completion is verified against a fixed, itemized gate list, not merely 'a PR exists')
- **A fleet is never reconciled or cleaned up merely because every worker has opened a PR; all completion gates must be met.**
  (trigger: every dispatched worker has opened a PR; outcome: PR existence alone never triggers reconciliation/cleanup)
- **In dispatch mode, monitor progress and reconcile merge order, PRs, and cross-repository implications.**
  (trigger: workers dispatched; outcome: cross-repository delivery is reconciled)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Verifying the itemized gate list per repository and reconciling dependency merge order.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J5 — governing constraint
- **A fleet is never reconciled merely because every worker opened a PR; all completion gates must be met.**

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when every itemized gate (pinned scope, validation, review artifacts, zero blocking findings, CI, resolved threads, dependency merge order) is verified per repository.

### Decision evidence
The per-repo gate verification is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
