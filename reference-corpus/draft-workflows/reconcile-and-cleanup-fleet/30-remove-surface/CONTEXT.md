# 30-remove-surface: remove surface

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-verify-handshakes/output/README.md | L4 | upstream artifact produced by `20-verify-handshakes` |

## Purpose

A resumable cleanup-phase record is published before and after; no process runs with its cwd inside the surface being removed.

Trigger (workflow-level): A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## What must become true here (durable outcome)

A resumable cleanup-phase record is published before and after; no process runs with its cwd inside the surface being removed.

## Behavior contract

- **Removing a worktree publishes a durable, resumable cleanup-phase record before the removal begins and updates it again once removal completes, so that a cleanup interrupted mid-removal can be safely retried later: the retry re-verifies exact identity of every recorded fact (owner repo, worker evidence, worktree Git identity) rather than assuming the prior attempt's state is still accurate.**
  (trigger: cleanup is retried after a prior invocation was interrupted mid-worktree-removal; outcome: a retried cleanup can always resume exactly where an interrupted attempt left off, without either repeating destructive work unsafely or losing track of what already happened)
  — `BU-P6-140`, `reference/sergeant-upstream/bin/sgt-cleanup` (L2621-2642)
- **Cleanup is only ever permitted to run when the worker's process cwd is verifiably not still inside the worktree being removed — verified via a system-wide process-working-directory scan (lsof) immediately before removal — so a worktree is never deleted while some process, tracked or untracked, still has it open.**
  (trigger: a worktree is about to be removed; outcome: a worktree removal can never proceed while any process — even one cleanup itself never launched or tracks — is still using it as its working directory)
  — `BU-P6-142`, `reference/sergeant-upstream/bin/sgt-cleanup` (L54-70, L268-270)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
