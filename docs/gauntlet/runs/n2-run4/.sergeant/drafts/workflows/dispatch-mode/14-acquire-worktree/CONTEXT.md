# 14-acquire-worktree

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the target branch already exists and may carry prior uncommitted-upstream work

**Outcome:** prior committed work is never silently overwritten or orphaned by a fresh dispatch; resuming it requires an explicit --adopt-branch

**Statement (the operative rule):** Re-dispatching onto an existing branch is blocked when that branch carries committed work not reachable from any remote, unless `--adopt-branch` is passed; the reachability test checks every remote, not just origin, so re-dispatch to a repository the operator cannot push to is not incorrectly blocked.

## What must become true here (durable outcome)

Prior committed work is never silently overwritten or orphaned by a fresh dispatch; resuming it requires an explicit --adopt-branch — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0292`: After acquiring a worktree, dispatch requires the worktree's `.git` to be a real (non-symlinked) file containing a well-formed `gitdir:` pointer, and resolves and records both the pointer and the canonicalized git directory, failing closed if either check does not hold.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0293`: Dispatch records the worktree's HEAD SHA at dispatch time (`initial_sha`) so workers and the reconcile path can later detect committed work added above that base.
- `BU-0925`: A branch's unpushed commits are determined by reachability from any configured remote-tracking ref, not by matching one specific remote branch name, so a commit that is published under a differently-named remote branch is correctly treated as already published.
- `BU-0926`: Unpushed-commit detection first confirms the local branch itself exists; for an absent branch it reports no unpushed commits rather than letting git raise a fatal error for a nonexistent ref.

