# 10-resolve-source-conflicts: resolve source conflicts

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-classify-and-locate/output/README.md | L4 | upstream artifact produced by `00-classify-and-locate` |

## Purpose

Where sources disagree, the answer follows the fixed precedence and the mismatch is reported as tracked work.

Trigger (workflow-level): The user asks what Sergeant is, how to install/configure/use it, where skills come from, or how to diagnose a Sergeant error.

## What must become true here (durable outcome)

Where sources disagree, the answer follows the fixed precedence and the mismatch is reported as tracked work.

## Behavior contract

- **When sources disagree, precedence is: command behavior/tests/supported --help output for released syntax, then AGENTS.md for always-on execution/safety policy, then the trigger-loaded skill for its own procedure, then docs/schema.md for project fields, then user documentation for walkthroughs.**
  (trigger: documentation sources disagree; outcome: disagreement is resolved by a fixed, principled precedence order rather than by whichever source was read last)
  — `BU-P5-122`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 45-50)
- **If observed command behavior differs from documentation, sergeant-help reports the mismatch, trusts tested/released behavior or supported --help output over the stale doc, and creates or suggests a documentation task.**
  (trigger: docs and observed behavior disagree; outcome: the operator gets both the correct current answer and a path to fixing the stale doc)
  — `BU-P5-127`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 72)
- **For flag or argument questions, sergeant-help runs --help only when the command supports it; otherwise it inspects the command's actual emitted usage/error contract and its tests.**
  (trigger: a flag or argument question is asked; outcome: the answer comes from the command's real, observed contract rather than assumed documentation)
  — `BU-P5-120`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 41-42)
- **For architectural questions where a configured Sergeant graph exists, sergeant-help runs graphify query and uses cited source locations in the answer.**
  (trigger: the question is architectural and a graph is configured; outcome: architectural answers are backed by cited, queryable graph evidence)
  — `BU-P5-119`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 39-40)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
