# 20-resolve-hunks: resolve hunks

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-research-intent/output/README.md | L4 | upstream artifact produced by `10-research-intent` |

## Purpose

Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted.

## Behavior contract

- **Each conflicting hunk is resolved by preserving both sides' intent where possible, or by picking the side matching the merge's stated goal and recording the trade-off when incompatible; resolution must never invent new behavior, and the merge/rebase must always be carried to completion rather than aborted.**
  (trigger: conflict intents are understood; outcome: every hunk is resolved consistent with recorded intent, with no invented behavior, and the operation is never abandoned via --abort)
  — `BU-P3-048`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 3, line 10)

## Helper: validate and finish (folded from demoted `30-validate` and `40-finish`, N1 adjudication A4)

`30-validate` and `40-finish` were classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 both are demoted and folded here, in sequence, as the concluding helper invocations of this checkpoint, subordinate to this stage's own judgment-bearing outcome:

- **After resolving conflicts, the actor discovers and runs the project's automated checks in the order typecheck, then tests, then format, fixing anything the merge broke.**
  (trigger: all hunks are resolved; outcome: the project's own automated checks pass and any merge-induced breakage is fixed)
  — `BU-P3-049`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 4, line 12)
- **The workflow concludes by staging and committing everything, and, if rebasing, continuing until every commit has been rebased.**
  (trigger: validation has passed; outcome: the merge or rebase is fully completed and committed)
  — `BU-P3-050`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 5, line 14)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
