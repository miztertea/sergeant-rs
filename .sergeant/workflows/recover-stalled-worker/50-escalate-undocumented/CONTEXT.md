# 50-escalate-undocumented: escalate undocumented

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-escalate-on-second-attempt/output/README.md | L4 | upstream artifact produced by `40-escalate-on-second-attempt` |

## Purpose

An undocumented/unrecognized stall class escalates rather than being guessed at.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

An undocumented/unrecognized stall class escalates rather than being guessed at.

## Behavior contract

- **When documentation does not cover an observed failure, the operator should use the sergeant-help skill to search existing docs first, then create a td task containing the exact reproduction, expected behavior, preserved state, and acceptance criteria.**
  (trigger: a failure is not covered by any existing troubleshooting entry; outcome: an undocumented failure always becomes a well-formed, reproducible, trackable task rather than being handled ad hoc and lost)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- An undocumented/unrecognized stall class is never guessed at — it always escalates.

### J2 — delegated to this stage
- How to conduct the `sergeant-help` documentation search and compose the `td` task's contents (reproduction, expected behavior, preserved state, acceptance criteria).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers — reaching this stage is itself the escalation for an undocumented stall class.

### Completion boundary
This stage may complete only once existing docs are searched via `sergeant-help` and a `td` task is created with exact reproduction, expected behavior, preserved state, and acceptance criteria.

### Decision evidence
The created `td` task is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
