# 09-reconcile-deliver: reconcile deliver

## Inputs

| File | Layer | Why |
|---|---|---|
| ../08-handle-decisions/output/README.md | L4 | upstream artifact produced by `08-handle-decisions` |

## Purpose

PRs, merge order, merges/deployments and cleanup eligibility are settled.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

PRs, merge order, merges/deployments and cleanup eligibility are settled.

## Behavior contract

- **Reconcile and deliver: surface PRs and merge order, complete approved merges/deployments, and run cleanup only after terminal state and preserved evidence are verified.**
  (trigger: execution and decisions are resolved; outcome: delivery is reconciled and cleanup is safe)
  — `BU-P1-034`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L146, step 9)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
