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
  — `BU-P5-070`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 108-114)
- **A fleet is never reconciled or cleaned up merely because every worker has opened a PR; all completion gates must be met.**
  (trigger: every dispatched worker has opened a PR; outcome: PR existence alone never triggers reconciliation/cleanup)
  — `BU-P5-071`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 115)
- **In dispatch mode, monitor progress and reconcile merge order, PRs, and cross-repository implications.**
  (trigger: workers dispatched; outcome: cross-repository delivery is reconciled)
  — `BU-P1-006`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L20, dispatch-mode step 3)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
