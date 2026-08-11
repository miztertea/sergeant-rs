# 00-interview-loop: interview loop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

One question at a time, waiting for each answer; discoverable facts are looked up rather than asked; each question carries a recommended answer.

Trigger (workflow-level): The user wants to stress-test their thinking, or uses a 'grill' trigger phrase.

## What must become true here (durable outcome)

One question at a time, waiting for each answer; discoverable facts are looked up rather than asked; each question carries a recommended answer.

## Behavior contract

- **The interview proceeds systematically down a decision tree, resolving dependent decisions in order, with the actor offering a recommended answer alongside each question.**
  (trigger: grilling workflow is active; outcome: every branch of the decision tree is walked and resolved with a recorded recommendation)
  — `BU-P3-006`, `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 6)
- **Within the interview loop, only one question is posed at a time, and the actor waits for the user's answer before asking the next.**
  (trigger: the interview loop is asking a question; outcome: one confirmed answer exists before the next question is posed)
  — `BU-P3-007`, `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 8)
- **The interview loop draws a firm line: facts discoverable by exploring the environment must be looked up by the actor; only genuine decisions are put to the user, and the actor waits for the user's answer on each.**
  (trigger: a question arises during the interview loop; outcome: facts are resolved autonomously; decisions are resolved only by explicit user answer)
  — `BU-P3-008`, `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 10)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Conflict X8 (synthesis.md §6): the identical "one question at a time, wait for the answer" shape is classified here as ordinary actor procedure but was separately raised as an engine-gap claim (G5) in the `sergeant-setup` partition. G5's resolution (a re-enterable `needs_input` stage) covers both cases without either needing to be a gap; this stage is written on that understanding, i.e. each unanswered question may end the stage in `needs_input` and re-enter it.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
