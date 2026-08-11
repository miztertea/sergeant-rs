# 00-read-source: read source

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The external skill's complete SKILL.md and referenced scripts are read before adopting it.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's complete SKILL.md and referenced scripts are read before adopting it.

## Behavior contract

- **Read the external skill's complete SKILL.md and referenced scripts before adopting it.**
  (trigger: vet-external-skill workflow entered; outcome: the skill's full instructions and scripts are read, not sampled)
  — `BU-P1-120`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L126, vet step 1)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
