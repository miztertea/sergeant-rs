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

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Determining which branch (logic vs. UI) applies, from the prompt, surrounding code, or the user directly (`BU-P3-012`, `BU-P3-013`).
- Choosing a heuristic fallback and recording the assumption when the branch is ambiguous and the user is unreachable (`BU-P3-014`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- The branch choice is ambiguous and the user *is* reachable — the heuristic fallback (`BU-P3-014`) applies only when the user cannot be reached; it is not a general-purpose shortcut.

### Completion boundary
This stage may complete only when exactly one branch (logic or UI) is selected and recorded.

### Decision evidence
The selected branch and, if a heuristic was used, the recorded assumption are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
