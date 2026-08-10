# 40-finish: finish

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-validate/output/README.md | L4 | upstream artifact produced by `30-validate` |

## Purpose

The merge/rebase is completed.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

The merge/rebase is completed.

## Behavior contract

- **The workflow concludes by staging and committing everything, and, if rebasing, continuing until every commit has been rebased.**
  (trigger: validation has passed; outcome: the merge or rebase is fully completed and committed)
  — `BU-P3-050`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 5, line 14)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
