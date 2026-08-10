# 02-check-queue: check queue

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-load-context/output/README.md | L4 | upstream artifact produced by `01-load-context` |

## Purpose

A matching tracked task is reused, or one is created because none is canonical.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

A matching tracked task is reused, or one is created because none is canonical.

## Behavior contract

- **Check the queue: run sgt-td-list and reuse a matching task in direct or dispatch mode; create a task only when no canonical task exists.**
  (trigger: context loaded; outcome: duplicate task creation is avoided)
  — `BU-P1-027`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L137, step 2)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
