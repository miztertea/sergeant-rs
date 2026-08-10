# 00-investigate: investigate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Primary sources only, every claim traced.

Trigger (workflow-level): A topic needs to be researched, or docs/API facts need gathering, and reading legwork is delegated.

## What must become true here (durable outcome)

Primary sources only, every claim traced.

## Behavior contract

- **Research must be conducted against primary sources (official docs, source code, specs, first-party APIs) rather than secondary summaries, with every claim traced back to its owning source.**
  (trigger: the research workflow is investigating; outcome: every claim in the findings traces to a primary source)
  — `BU-P3-042`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 1, line 10)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
