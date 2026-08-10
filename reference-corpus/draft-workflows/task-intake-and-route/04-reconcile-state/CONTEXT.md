# 04-reconcile-state: reconcile state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-choose-mode/output/README.md | L4 | upstream artifact produced by `03-choose-mode` |

## Purpose

Active workers, branches, worktrees, retained gates and handoffs are inspected; preserved work is resumed rather than duplicated.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Active workers, branches, worktrees, retained gates and handoffs are inspected; preserved work is resumed rather than duplicated.

## Behavior contract

- **Reconcile existing state: run sgt-watch --sync-all, then inspect active workers, branches, worktrees, retained gates, and handoffs before starting; resume or take over preserved work rather than creating duplicates.**
  (trigger: an execution mode is chosen; outcome: no preserved work is duplicated or abandoned)
  — `BU-P1-029`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L139, step 4)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
