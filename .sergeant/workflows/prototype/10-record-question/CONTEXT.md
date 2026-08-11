# 10-record-question: record question

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-select-branch/output/README.md | L4 | upstream artifact produced by `00-select-branch` |

## Purpose

The design question the prototype must answer is recorded.

Trigger (workflow-level): The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## What must become true here (durable outcome)

The design question the prototype must answer is recorded.

## Behavior contract

- **Before any code is written, the actor records the state model and the exact question the prototype answers, so the question can be checked against the eventual result even if the user returns to it later.**
  (trigger: the logic-prototype branch has been selected; outcome: a written statement of the question and state model exists before code is written)
  — `BU-P3-021`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 1, line 18)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
