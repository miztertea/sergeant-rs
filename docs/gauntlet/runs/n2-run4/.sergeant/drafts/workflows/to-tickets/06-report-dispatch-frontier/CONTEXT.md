# 06-report-dispatch-frontier

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-validate-published-graph/output/outcome.md | L4 | upstream evidence produced by `validate-published-graph` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** publishing has completed and the frontier is being computed

**Outcome:** only tickets with no remaining blockers are reported as immediately dispatchable

**Statement (the operative rule):** The dispatch frontier reported after publishing consists of the tickets that have no unfinished blockers.

## What must become true here (durable outcome)

Only tickets with no remaining blockers are reported as immediately dispatchable — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1331`: Recommended concurrency defaults to one worker per owning repository, unless the project explicitly supports more.
- `BU-1332`: Dispatch does not happen unless the user asked to begin implementation.

