# 20-answer-or-hand-off: answer or hand off

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-resolve-source-conflicts/output/README.md | L4 | upstream artifact produced by `10-resolve-source-conflicts` |

## Purpose

Either a fixed-format answer with command, preconditions, evidence and doc links, or an explicit hand-off to the owning procedure.

Trigger (workflow-level): The user asks what Sergeant is, how to install/configure/use it, where skills come from, or how to diagnose a Sergeant error.

## What must become true here (durable outcome)

Either a fixed-format answer with command, preconditions, evidence and doc links, or an explicit hand-off to the owning procedure.

## Behavior contract

- **Answers state the exact command, required preconditions, expected evidence, and links to repository-relative documentation paths.**
  (trigger: an answer is being formulated; outcome: every answer is independently actionable and independently verifiable)
  — `BU-P5-121`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 43-44)
- **Destructive operations are kept out of examples unless the documentation itself requires confirmation for them and the user explicitly requested them.**
  (trigger: an example command is being included in an answer; outcome: help output never casually demonstrates a destructive action)
  — `BU-P5-125`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 64-65)
- **If a question actually requires project ownership context, sergeant-help loads load-project and runs sgt-context rather than answering from memory.**
  (trigger: a question needs project-specific context; outcome: help defers to the workflow that actually resolves that context)
  — `BU-P5-128`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 73)
- **If a question actually requires implementation or fleet mutation, sergeant-help hands off to the owning procedural skill; help itself remains strictly read-only.**
  (trigger: a question actually requires a mutating action; outcome: sergeant-help never performs or triggers mutation itself)
  — `BU-P5-129`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 74)
- **sergeant-help answers Sergeant installation, setup, usage, skills, and troubleshooting questions strictly from repository-owned documentation.**
  (trigger: a Sergeant informational question is asked; outcome: the answer is grounded in repository-owned docs, not invented)
  — `BU-P5-113`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 3-4)
- **sergeant-help is loaded when the user asks what Sergeant is, how to install/configure/use it, where skills come from, how to run a command/workflow, or how to diagnose a Sergeant error.**
  (trigger: one of the listed question types is asked; outcome: the read-only help workflow, not an executing workflow, is selected)
  — `BU-P5-114`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 7-8)
- **sergeant-help is never used as a substitute for load-project, cross-repo-work, dispatch, or wiki once the user has actually requested execution of those procedures.**
  (trigger: the user has requested execution, not just an explanation; outcome: read-only help never silently absorbs a request meant for an executing workflow)
  — `BU-P5-115`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 12-13)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
