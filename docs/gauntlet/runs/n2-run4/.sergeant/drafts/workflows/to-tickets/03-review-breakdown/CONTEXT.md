# 03-review-breakdown

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-draft-tickets/output/outcome.md | L4 | upstream evidence produced by `draft-tickets` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** publication has not been explicitly requested immediately

**Outcome:** the user reviews the breakdown before any ticket is actually published

**Statement (the operative rule):** Unless the user explicitly said to create or publish tickets immediately, the proposed breakdown is presented first, before publishing.

## What must become true here (durable outcome)

The user reviews the breakdown before any ticket is actually published — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1321`: When confirming the breakdown, the skill asks only whether granularity, ownership, and blocking edges are correct, and does not ask the user to reconfirm decisions already made.

