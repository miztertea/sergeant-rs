# 10-research-intent: research intent

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-assess-state/output/README.md | L4 | upstream artifact produced by `00-assess-state` |

## Purpose

The intent behind each conflicting side is researched.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

The intent behind each conflicting side is researched.

## Behavior contract

- **For each conflict, the actor traces the original intent behind each side's change via commit messages, PRs, and issues/tickets before attempting resolution.**
  (trigger: conflicting hunks have been identified; outcome: the intent behind each conflicting change is understood before it is resolved)
  — `BU-P3-047`, `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 2, line 8)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
