# 60-reconcile: reconcile

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-handoff-or-stop/output/README.md | L4 | upstream artifact produced by `50-handoff-or-stop` |

## Purpose

PR URLs, heads, CI, review threads, merge and deployment order, terminal task/fleet state.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

PR URLs, heads, CI, review threads, merge and deployment order, terminal task/fleet state.

## Behavior contract

- **After dispatched workers finish, cross-repo-work reconciles PR URLs and final heads, required CI and unresolved review threads, merge order from dependency edges, deployment order and cross-repo release notes, and terminal td/fleet state and cleanup eligibility.**
  (trigger: dispatch has completed for the plan's repositories; outcome: the multi-repo outcome is reconciled against every planned gate, not just individual PR existence)
  — `BU-P5-052`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 87-93)
- **cross-repo-work never reports the cross-repo outcome complete until every owning repository has a terminal result or an explicit preserved blocker.**
  (trigger: reconciliation is being evaluated for completeness; outcome: partial completion is never reported as done)
  — `BU-P5-053`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 95-96)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
