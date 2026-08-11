# 23-reconcile-incomplete-dispatch

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a dispatched repo's grace period expires and no owned live pane can be found

**Outcome:** evidence of committed work is surfaced to the operator rather than silently discarded behind a generic failure message

**Statement (the operative rule):** When an in-progress dispatch's grace period expires with no owned live pane, the interactive fleet-watch loop checks the worktree for commits made above the dispatch base; if any exist, it marks the repo failed with a message directing the operator to reconcile the preserved branch before re-dispatch, rather than the generic 'no worktree or pane acquired' failure.

## What must become true here (durable outcome)

Evidence of committed work is surfaced to the operator rather than silently discarded behind a generic failure message — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0589`: The grace period before an incomplete dispatch is considered expired is configurable via SERGEANT_DISPATCH_GRACE_SECONDS (default 300) and the interactive fleet-watch loop dies if that value is not a non-negative integer, rather than silently falling back to a default.
- `BU-0594`: The interactive fleet-watch loop's committed-work log for a dispatch prefers the range from the dispatch-recorded initial_sha to HEAD, and only falls back to diffing against the branch's upstream tracking ref for fleet state that predates initial_sha being recorded; it produces nothing (not an error) when git is unavailable or no upstream is configured.

