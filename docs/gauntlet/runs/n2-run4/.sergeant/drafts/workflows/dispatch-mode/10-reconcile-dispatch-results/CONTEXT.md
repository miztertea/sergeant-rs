# 10-reconcile-dispatch-results

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a worker has opened a PR and other completion evidence looks satisfied

**Outcome:** dependency-gate satisfaction is a separate, required condition for done, not implied by other evidence

**Statement (the operative rule):** A worker is not considered done until its dependency gate is satisfied, even if merge order among dependent repos would otherwise suggest completion.

## What must become true here (durable outcome)

Dependency-gate satisfaction is a separate, required condition for done, not implied by other evidence — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0277`: A fleet is never reconciled or cleaned up merely because every worker has opened a PR; all completion gates must be met.

