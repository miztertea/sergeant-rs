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
  — `BU-P2-060`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Task-first mode step 1, lines 36-38)
- **The actor makes the changes the task describes and commits them on a feature branch; if the user is on the repository's default branch, a feature branch must be created first, because the gate validates committed history on a non-default branch.**
  (trigger: the task's changes have been made; outcome: the work lands as a commit on a non-default feature branch before validation can proceed)
  — `BU-P2-061`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Task-first mode step 2, lines 39-42)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Isolating exactly which working-tree changes belong to the task (versus pre-existing unrelated changes) requires judgment, not a mechanical diff.

## Additional note

Restored per N1 adjudication A5 (finding N1-BH-04) — see `00-check-scope`'s Additional note for the full restoration rationale and the entry-point relationship to `20-select-intent-transport`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
