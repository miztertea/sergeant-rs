# 20L-build-logic: build logic

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-record-question/output/README.md | L4 | upstream artifact produced by `10-record-question` |

## Purpose

A logic prototype is built to answer the recorded question.

Trigger (workflow-level): The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## What must become true here (durable outcome)

A logic prototype is built to answer the recorded question.

## Behavior contract

- **The logic-prototype branch builds a small interactive terminal app so the user can hand-drive a state model through the cases that are hard to evaluate on paper.**
  (trigger: the branch question is about business logic, state transitions, or data shape; outcome: an interactive terminal app exists that lets the user push the state model through concrete cases)
  — `BU-P3-020`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (header, line 3)
- **The logic under test must be isolated behind a small, pure interface that can later be lifted into the real codebase; only the terminal UI shell around it is truly throwaway.**
  (trigger: building the logic-prototype; outcome: the validated logic module is reusable independent of the throwaway TUI)
  — `BU-P3-022`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 3 intro, line 28)
- **The logic module must stay pure — no I/O, no terminal code, no console output used for control flow — and the dependency direction is one-way: the TUI imports the logic module, never the reverse.**
  (trigger: implementing the isolated logic module; outcome: the logic module has no dependency on the TUI shell)
  — `BU-P3-023`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 3, line 37)
- **The terminal UI re-renders the full frame from scratch on every update rather than appending output, so the user always sees one stable current view.**
  (trigger: building the interactive terminal shell; outcome: the terminal shows one current frame, never an accumulating scrollback)
  — `BU-P3-024`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 4, line 43)
- **After each user action, the shell replaces the displayed frame entirely rather than appending to it.**
  (trigger: a user action has just been dispatched; outcome: the displayed frame reflects only the current state, not a history of prior frames)
  — `BU-P3-025`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 4, line 54)
- **The logic prototype is wired into the host project's existing task runner so it can be started by name rather than by remembering a file path.**
  (trigger: the logic prototype is ready to hand to the user; outcome: the prototype is runnable via the project's normal task-runner invocation)
  — `BU-P3-026`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 5, line 61)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Conditional: entered only when `00-select-branch` selected the logic branch. Mutually exclusive with `20U-build-variants`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
