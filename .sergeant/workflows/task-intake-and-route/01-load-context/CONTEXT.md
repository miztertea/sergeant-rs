# 01-load-context: load context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Owning repositories, inherited instructions and cross-repo dependencies are known.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Owning repositories, inherited instructions and cross-repo dependencies are known.

## Behavior contract

- **Load context: run sgt-context and identify the owning repository or repositories, inherited instructions, configured paths, and cross-repository dependencies before selecting an execution mode.**
  (trigger: a task has been brought; outcome: project/repository context is established before mode selection)
  — `BU-P1-026`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L136, step 1)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **load-project** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
