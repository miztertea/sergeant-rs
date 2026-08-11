# 00-select-branch: select branch

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Which question type (logic vs. UI) is decided; a heuristic fallback is recorded when the user is unreachable.

Trigger (workflow-level): The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## What must become true here (durable outcome)

Which question type (logic vs. UI) is decided; a heuristic fallback is recorded when the user is unreachable.

## Behavior contract

- **The first checkpoint of the prototype workflow is determining which of the two question-types (logic/state vs. UI) is being asked, using the prompt, surrounding code, or the user directly.**
  (trigger: prototype workflow invoked; outcome: one of the two branches (logic or UI) is selected)
  — `BU-P3-012`, `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (line 12)
- **When the question is about appearance, the workflow routes to the UI-prototype branch, which produces several structurally different UI variants switchable in-browser.**
  (trigger: the question is about what something should look like; outcome: the UI-prototype branch is selected and its variant/switcher shape is set as the target artifact)
  — `BU-P3-013`, `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (line 15)
- **When the branch choice is ambiguous and the user cannot be reached, the workflow falls back to a heuristic based on the surrounding code's shape and records the assumption explicitly in the prototype, rather than blocking.**
  (trigger: the branch question is ambiguous and the user is unreachable; outcome: a branch is chosen by heuristic and the assumption is recorded, rather than the workflow stalling)
  — `BU-P3-014`, `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (line 17)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
