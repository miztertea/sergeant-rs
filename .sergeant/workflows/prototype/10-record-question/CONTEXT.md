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

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Phrasing the state model and question in the form later checkable against the eventual result.

### J1 — local choices allowed
- Where the written statement lives (README, top-of-file comment).

### J0 — must become `needs_input`
- The question cannot be stated precisely enough to be later checked against a result — an unstated question is "pure waste" per the source's own framing; ask rather than proceed on a vague one.

### Completion boundary
This stage may complete only when a written statement of the question and state model exists, before any code is written.

### Decision evidence
The recorded question/state-model statement is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
