# 00-assess-state: assess state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The current merge/rebase state is assessed.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

The current merge/rebase state is assessed.

## Behavior contract

- **The first checkpoint establishes the current merge/rebase state by inspecting git history and the conflicting files.**
  (trigger: the workflow begins; outcome: the actor has an accurate picture of what is conflicting and why)
  — `BU-P3-046`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 1, line 6)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
