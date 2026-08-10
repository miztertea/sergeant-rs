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

## Helper: record outcomes (folded from demoted `07-record-outcomes`, N1 adjudication A4)

`07-record-outcomes` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as the concluding helper invocation of this checkpoint, subordinate to this stage's own judgment-bearing outcome:

- **In direct mode, record handoff, PR, merge, deployment, and cleanup outcomes.**
  (trigger: delivery complete; outcome: delivery outcomes are durably recorded)
  — `BU-P1-014`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L36, direct-mode handoff step)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
