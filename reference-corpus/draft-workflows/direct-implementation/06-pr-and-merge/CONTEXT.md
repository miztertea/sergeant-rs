# 06-pr-and-merge: pr and merge

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../05-shipping-gate/output/README.md | L4 | upstream artifact produced by `05-shipping-gate` |

## Purpose

A PR is opened and merged per repository convention.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

A PR is opened and merged per repository convention.

## Behavior contract

- **In direct mode, open a PR for every implementation and satisfy required CI, review threads, and merge authorization before calling delivery complete.**
  (trigger: validation passed; outcome: delivery is not declared complete until PR/CI/review/merge conditions are met)
  — `BU-P1-013`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L34-35, direct-mode PR step)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
