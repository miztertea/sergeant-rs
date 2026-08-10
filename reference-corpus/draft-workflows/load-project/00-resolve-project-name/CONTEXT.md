# 00-resolve-project-name: resolve project name

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

An exact registered project name is bound, or the run stops asking whether to register.

Trigger (workflow-level): A project is named, registered, edited, synced, or listed; or repository ownership is not already established.

## What must become true here (durable outcome)

An exact registered project name is bound, or the run stops asking whether to register.

## Behavior contract

- **If the project name is unknown, load-project runs sgt-list and requires an exact registered name before proceeding.**
  (trigger: the project name is not already known; outcome: work never proceeds against a guessed or partial project name)
  — `BU-P5-092`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 17-18)
- **If a named project is unregistered, load-project stops and asks whether to register it, rather than proceeding on an assumed project.**
  (trigger: the requested project is unregistered; outcome: no work proceeds against a project that does not yet exist)
  — `BU-P5-108`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 72)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
