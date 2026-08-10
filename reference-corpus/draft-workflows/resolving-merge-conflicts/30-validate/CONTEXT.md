# 30-validate: validate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-resolve-hunks/output/README.md | L4 | upstream artifact produced by `20-resolve-hunks` |

## Purpose

Typecheck, tests, format run in that order.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

Typecheck, tests, format run in that order.

## Behavior contract

- **After resolving conflicts, the actor discovers and runs the project's automated checks in the order typecheck, then tests, then format, fixing anything the merge broke.**
  (trigger: all hunks are resolved; outcome: the project's own automated checks pass and any merge-induced breakage is fixed)
  — `BU-P3-049`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 4, line 12)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
