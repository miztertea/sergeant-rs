# 05-validate-published-graph

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-publish-tickets/output/outcome.md | L4 | upstream evidence produced by `publish-tickets` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** publication has completed

**Outcome:** the dependency graph is checked to be free of cycles and fabricated cross-repo edges before being considered valid

**Statement (the operative rule):** After publishing, the skill confirms no circular or cross-repo pseudo-dependencies exist in the ticket graph.

## What must become true here (durable outcome)

The dependency graph is checked to be free of cycles and fabricated cross-repo edges before being considered valid — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1329`: Stale duplicate tickets are closed only with an explicit superseding task, via the task tracker.

