# 01-conduct-research

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a topic needs primary-source research

**Outcome:** research proceeds in parallel with the invoking actor's other work instead of blocking it

**Statement (the operative rule):** A background agent is spun up to do primary-source research so the invoking actor keeps working while it reads.

## What must become true here (durable outcome)

Research proceeds in parallel with the invoking actor's other work instead of blocking it — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0979`: The background research agent investigates against primary sources — official docs, source code, specs, first-party APIs — not secondary write-ups, following every claim back to the source that owns it.
- `BU-0980`: The research findings are written to a single Markdown file, citing each claim's source.
- `BU-0981`: The findings file is saved where the repo already keeps such notes, matching the existing convention; if there is no existing convention, it is placed somewhere sensible and the location is stated.

