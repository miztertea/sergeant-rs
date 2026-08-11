# 01-load-task-context: load task context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |

## Purpose

The task's originating context is loaded and understood.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

The task's originating context is loaded and understood.

## Behavior contract

- **In direct mode, run sgt-context for the project and td context for the owning task before making any edit.**
  (trigger: direct mode selected; outcome: task and repository context are loaded before mutation)
  — `BU-P1-008`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L24-25, direct-mode step 1)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
