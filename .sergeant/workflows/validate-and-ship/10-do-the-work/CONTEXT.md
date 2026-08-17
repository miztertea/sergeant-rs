# 10-do-the-work: do the work

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-check-scope/output/README.md | L4 | upstream artifact produced by `00-check-scope` |

## Purpose

In task-first mode, the described task is carried out and committed on a feature branch before validation begins.

Trigger (workflow-level, direct-invocation entry): `00-check-scope` determined task-first mode.

## What must become true here (durable outcome)

Only the task's own changes are committed, on a non-default feature branch, with unrelated pre-existing uncommitted changes left untouched.

## Behavior contract

- **In task-first mode, before changing or committing anything the actor inspects `git status`, preserves unrelated pre-existing uncommitted changes, and when committing, commits only the changes belonging to the user's task.**
  (trigger: task-first mode is entered; outcome: only the task's own changes are committed; unrelated pre-existing changes survive untouched)
- **The actor makes the changes the task describes and commits them on a feature branch; if the user is on the repository's default branch, a feature branch must be created first, because the gate validates committed history on a non-default branch.**
  (trigger: the task's changes have been made; outcome: the work lands as a commit on a non-default feature branch before validation can proceed)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Isolating exactly which working-tree changes belong to the task versus pre-existing unrelated changes, and committing only the former.
- Deciding whether a feature branch must be created first (the user was on the repository's default branch) or already exists.

### J1 — local choices allowed
- Commit message wording, provided scope is correctly isolated.

### J0 — must become `needs_input`
- A working-tree change cannot be confidently attributed to the task or to a pre-existing unrelated edit — guessing risks committing someone else's in-progress work.

### Completion boundary
This stage may complete only when the task's own changes, and only those, are committed on a non-default feature branch.

### Decision evidence
The commit itself is the durable record of scope; no separate decision log.

## Additional note

Restored per N1 adjudication A5 (finding N1-BH-04) — see `00-check-scope`'s Additional note for the full restoration rationale and the entry-point relationship to `20-select-intent-transport`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
