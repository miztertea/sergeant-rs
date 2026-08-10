# 40-reconcile-before-launch: reconcile before launch

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-create-tracked-work/output/README.md | L4 | upstream artifact produced by `30-create-tracked-work` |

## Purpose

Bulk fleet reconciliation runs before new work is created.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Bulk fleet reconciliation runs before new work is created.

## Behavior contract

- **Bulk fleet reconciliation syncs worktree status into fleet state, stops only identity-verified done or failed worker processes, and marks an interrupted dispatched record failed only once it has had neither a worktree nor an owned live process for a default 300-second grace period (configurable), while it always preserves needs_input, blocked, and orphaned records, and dispatch always runs this reconciliation automatically before creating new work.**
  (trigger: sgt-watch --sync-all runs, or dispatch runs it automatically before new work; outcome: fleet state converges toward truth using identity-verified evidence and a bounded grace period, never a bare liveness guess, and never silently sweeps a needs_input/blocked/orphaned record)
  — `BU-P8-070`, `reference/sergeant-upstream/docs/using-sergeant.md` (L137-155 (Monitor work))

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
