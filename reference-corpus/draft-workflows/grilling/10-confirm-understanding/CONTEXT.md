# 10-confirm-understanding: confirm understanding

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-interview-loop/output/README.md | L4 | upstream artifact produced by `00-interview-loop` |

## Purpose

An explicit user confirmation gate before any action.

Trigger (workflow-level): The user wants to stress-test their thinking, or uses a 'grill' trigger phrase.

## What must become true here (durable outcome)

An explicit user confirmation gate before any action.

## Behavior contract

- **The workflow may not proceed to action until the user explicitly confirms shared understanding has been reached.**
  (trigger: the interview loop has walked all decision-tree branches; outcome: an explicit user confirmation of shared understanding exists before any action is taken)
  — `BU-P3-009`, `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 12)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
