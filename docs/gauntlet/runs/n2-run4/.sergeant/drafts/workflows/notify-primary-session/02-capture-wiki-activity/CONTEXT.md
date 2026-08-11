# 02-capture-wiki-activity

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../01-publish-notification/output/outcome.md | L4 | upstream evidence produced by `publish-notification` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the notify step is invoked

**Outcome:** a durable, searchable activity record exists for every update regardless of the transport outcome

**Statement (the operative rule):** Every notify call writes a wiki activity entry recording the task id, event class, full message text, and any extracted PR link — independent of which transport delivered, or failed to deliver, the update.

## What must become true here (durable outcome)

A durable, searchable activity record exists for every update regardless of the transport outcome — per the Statement above, which is the operative rule this stage exists to enforce.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0690`: The wiki activity entry for an update extracts and links the first GitHub PR URL found in the message text, if any.

