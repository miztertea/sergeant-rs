# 00-require-terminal: require terminal

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Every targeted repo is safely terminal and the owning task is verifiably closed; "not closed" is distinguished from "could not be looked up".

Trigger (workflow-level): A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## What must become true here (durable outcome)

Every targeted repo is safely terminal and the owning task is verifiably closed; "not closed" is distinguished from "could not be looked up".

## Behavior contract

- **Cleanup of a completed task is a bounded procedure that removes every repo's worktree (returning a treehouse lease or removing a plain git worktree) and, only when every repo is being cleaned at once, retires the fleet state directory itself — refusing to run at all unless every targeted repo's status is safely terminal, requiring the owning tracked-work task to be closed as well.**
  (trigger: a task's repos have all reached a safely terminal state and the operator wants to reclaim resources; outcome: worktrees are reclaimed and fleet state is retired only when every safety precondition (terminal status, closed tracked-work, response-handshake completeness) holds)
  — `BU-P6-135`, `reference/sergeant-upstream/bin/sgt-cleanup` (L2-7)
- **Cleanup refuses to remove a worktree unless the owning tracked-work task is verifiably closed — and it distinguishes 'not closed yet' from 'could not even be looked up', reporting the infrastructural failure by itself rather than letting an unreadable task tracker silently masquerade as 'not terminal'.**
  (trigger: cleanup is checking whether a repo's tracked work is done before removing its worktree; outcome: a diagnosability failure (couldn't check) is never confused with a real safety refusal (checked, and it's not closed))
  — `BU-P6-136`, `reference/sergeant-upstream/bin/sgt-cleanup` (L988-992, L1028-1042)
- **Fleet cleanup requires terminal/reconciled state, configured cleanup-owner proof for the repository/worktree or treehouse lease, preserved evidence, explicit cleanup-phase proof for a replayed removal or an already-absent worktree, fully acknowledged response transport, and no uncommitted or in-use worktree state; cleanup must never be used to resolve a waiting, blocked, or orphaned worker.**
  (trigger: sgt-cleanup is invoked for a fleet task; outcome: destructive cleanup only ever proceeds once every one of these conditions holds, and cleanup is never a shortcut for actually resolving unfinished work)
  — `BU-P8-092`, `reference/sergeant-upstream/docs/using-sergeant.md` (L399-408 (Clean completed fleet state))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
