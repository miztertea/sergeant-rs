# 04-publish-tickets

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-review-breakdown/output/outcome.md | L4 | upstream evidence produced by `review-breakdown` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** tickets and their parent epics are being published

**Outcome:** epics exist with real IDs before any child ticket that references them is created

**Statement (the operative rule):** Local epics are created first via the task tracker, so that child tickets can reference real epic IDs.

## What must become true here (durable outcome)

Epics exist with real IDs before any child ticket that references them is created — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1323`: Tickets are created in dependency order, blockers first.
- `BU-1324`: For an existing task, the skill updates it rather than creating a duplicate.
- `BU-1325`: The task-tracker creation step is used when one approved logical outcome needs matching task records in several registered repositories, with repository-specific details then added via the task tracker.
- `BU-1326`: Because the task tracker dependencies are repository-local, cross-repository blockers are represented by recording the counterpart repo and the task tracker ID in both descriptions or logs and stating the exact merge order, never by inventing a native dependency edge the task tracker cannot enforce across databases.
- `BU-1327`: Newly published tasks are not marked `in_progress` by this skill; they remain `open` until dispatch or a worker starts them.

