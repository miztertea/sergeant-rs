# 02-reconcile-existing-state: reconcile existing state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../01-load-task-context/output/README.md | L4 | upstream artifact produced by `01-load-task-context` |

## Purpose

Existing branch/worktree/handoff state is inspected and resumed rather than duplicated.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

Existing branch/worktree/handoff state is inspected and resumed rather than duplicated.

## Behavior contract

- **In direct mode, reconcile existing workers and preserved worktrees before editing; never duplicate or race work already in progress.**
  (trigger: task context loaded; outcome: no duplicate or racing work is created)
  — `BU-P1-009`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L26-27, direct-mode step 2)
- **Before editing anything in direct mode, existing worktrees and workers for the same owning repository/task must be reconciled.**
  (trigger: direct mode has loaded context and is about to begin editing; outcome: conflicting or duplicate in-flight work is discovered and resolved before new edits compound it)
  — `BU-P8-056`, `reference/sergeant-upstream/docs/using-sergeant.md` (L23 (Direct mode, step 2))

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
