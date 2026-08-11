# 00-gather-context: gather context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Context is synthesized, never gathered by interview.

Trigger (workflow-level): A design needs to be turned into a spec-shaped ticket before implementation.

## What must become true here (durable outcome)

Context is synthesized, never gathered by interview.

## Behavior contract

- **Turning the current conversation into a published spec is a synthesis-only procedure: do not interview the user, and instead write the spec from what has already been discussed and from codebase exploration.**
  (trigger: the user asks to turn the current conversation into a spec/PRD; outcome: a spec is published to the issue tracker without a separate interview pass)
  — `BU-P4-050`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (L3, L7)
- **Before drafting a spec, explore the repository to understand current state (if not already done), and use the project's domain glossary vocabulary and respect any ADRs in the touched area throughout the spec.**
  (trigger: a spec is about to be drafted; outcome: the spec is grounded in current codebase state and consistent with the project's glossary/ADRs)
  — `BU-P4-051`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 1, L13)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
