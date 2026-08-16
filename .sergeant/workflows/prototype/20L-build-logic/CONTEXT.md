# 20L-build-logic: build logic

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-record-question/output/README.md | L4 | upstream artifact produced by `10-record-question` |
| ../references/shared-rules.md | L3 | rules that apply to both branches (throwaway marking/location, one command to run, no persistence by default, surface the state) — added ICM-R3, closing a gap where these were extracted at N1 but never materialized (`BU-PROTO-28`) |

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

## Bounded judgment

Apply `@@bounded-judgment`. See `../references/shared-rules.md` for the rules this stage shares with `20U-build-variants`.

### J2 — delegated to this stage
- Designing the pure logic module's interface and the TUI shell around it (`BU-P3-022`, `BU-P3-023`).

### J1 — local choices allowed
- Terminal rendering details, provided the frame is fully replaced on every update, never appended (`BU-P3-024`, `BU-P3-025`).

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when an interactive terminal app exists, the logic module is pure and one-way-dependent (TUI imports logic, never the reverse), and the prototype is wired into the project's task runner (`BU-P3-026`) — plus the shared rules in `../references/shared-rules.md`.

### Decision evidence
The built prototype and its interface design are this stage's own durable output.

## Additional note

Conditional: entered only when `00-select-branch` selected the logic branch. Mutually exclusive with `20U-build-variants`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
