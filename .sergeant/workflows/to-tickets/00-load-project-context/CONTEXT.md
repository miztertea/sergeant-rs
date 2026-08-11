# 00-load-project-context: load project context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Project context is loaded.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

Project context is loaded.

## Behavior contract

- **When loading project context for ticket authoring, do not automatically add td instructions to a repository's own guidance files as a side effect.**
  (trigger: project context is being loaded, and the repo lacks td instructions; outcome: the repository's own guidance files are left untouched unless explicitly requested)
  — `BU-P4-064`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Load Project Context, L46)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **load-project** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
