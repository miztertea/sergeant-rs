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

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
