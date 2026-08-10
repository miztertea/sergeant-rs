# 40-define-delivery-gates: define delivery gates

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-inspect-repository-state/output/README.md | L4 | upstream artifact produced by `30-inspect-repository-state` |

## Purpose

Per-repo gate: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

Per-repo gate: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions.

## Behavior contract

- **Every per-repository delivery gate must include: the owning td task (or its creation requirement), the fixed point and preserved source state, repository-specific test/lint/typecheck/build commands, Standards and Spec review sources, PR dependency and deployment order, and any already-approved or still-missing data/security/destructive decisions.**
  (trigger: delivery gates are being defined per repository; outcome: every repository's brief has a complete, checkable gate set before dispatch)
  — `BU-P5-049`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 65-74)
- **The cross-repo plan is complete only when every owning repository has one implementation brief, acceptance evidence, and an acyclic dependency position.**
  (trigger: delivery gates have been drafted for every repository; outcome: the plan's completion condition is explicit and checkable)
  — `BU-P5-050`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 76-77)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
