# 00-resolve-output-path: resolve output path

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

One project-level output path is confirmed (or requested from the user) and is outside every source repo.

Trigger (workflow-level): Architecture work needs whole-project structure, or the operator asks for a graph/refresh.

## What must become true here (durable outcome)

One project-level output path is confirmed (or requested from the user) and is outside every source repo.

## Behavior contract

- **A single project-level graphify.output path, located outside any source repository, is configured when project Graphify is required.**
  (trigger: project Graphify is required; outcome: graph output never lands inside a source repository)
  — `BU-P5-100`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 42-43)
- **Generated project graph output is never published inside an owning source repository.**
  (trigger: Graphify has run successfully; outcome: graph output stays a separate, disposable projection, never mixed into source history)
  — `BU-P5-107`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 66)
- **A recurring or recursive graphify output is diagnosed by inspecting the project's configured graphify.output path and keeping exactly one output per project, located outside every source repository, rather than regenerating or moving an existing graph without first confirming the intended global per-project path.**
  (trigger: a project's published graph looks wrong or appears to recursively include itself; outcome: the operator confirms and fixes the configured output location rather than blindly regenerating)
  — `BU-P8-103`, `reference/sergeant-upstream/docs/troubleshooting.md` (L148-152 (Graphify output is wrong or recursive))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
