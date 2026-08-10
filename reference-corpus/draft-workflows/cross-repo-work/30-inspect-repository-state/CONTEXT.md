# 30-inspect-repository-state: inspect repository state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-define-dependency-order/output/README.md | L4 | upstream artifact produced by `20-define-dependency-order` |

## Purpose

Non-main branches, uncommitted changes, ahead/behind, worktrees, preserved workers recorded without mutating anything.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

Non-main branches, uncommitted changes, ahead/behind, worktrees, preserved workers recorded without mutating anything.

## Behavior contract

- **cross-repo-work runs sgt-status <project> and records non-main branches, uncommitted changes, ahead/behind state, active worktrees, and preserved workers for every owning repository before planning proceeds.**
  (trigger: ownership and dependencies are being established; outcome: the plan accounts for each owning repository's actual current state)
  — `BU-P5-047`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 56-59)
- **cross-repo-work never stashes, resets, switches, or cleans repository state during planning; it either routes an existing canonical branch/worktree into the worker brief or stops for a decision when state conflicts with the requested outcome.**
  (trigger: planning inspects a repository with pre-existing state; outcome: planning is strictly read-only with respect to repository state)
  — `BU-P5-048`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 61-63)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
