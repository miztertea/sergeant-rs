# 40-retire-state: retire state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-remove-surface/output/README.md | L4 | upstream artifact produced by `30-remove-surface` |

## Purpose

Whole-task state is retired only when every repo is cleaned together.

Trigger (workflow-level): A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## What must become true here (durable outcome)

Whole-task state is retired only when every repo is cleaned together.

## Behavior contract

- **Cleanup only ever retires the whole fleet-state directory for a task when every repo is being cleaned together (no repo filter given); a single-repo-scoped cleanup invocation only ever removes that one repo's worktree and never touches the shared task-level fleet state.**
  (trigger: cleanup is invoked, optionally scoped to a single repo; outcome: a task's shared fleet-level state (brief, intent, notifications) is only ever retired once every one of its repos has actually been cleaned up)
  — `BU-P6-141`, `reference/sergeant-upstream/bin/sgt-cleanup` (L2749-2751)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
