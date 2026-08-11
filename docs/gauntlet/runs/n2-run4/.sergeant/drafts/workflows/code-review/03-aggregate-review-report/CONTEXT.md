# 03-aggregate-review-report

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-prepare-review-inputs/output/outcome.md | L4 | upstream evidence produced by `prepare-review-inputs` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** both sub-agent reports have returned

**Outcome:** the two axes stay visibly separate in the final report

**Statement (the operative rule):** The two sub-agent reports are presented verbatim (or lightly cleaned) under separate `## Standards` and `## Spec` headings, never merged or reranked against each other.

## What must become true here (durable outcome)

The two axes stay visibly separate in the final report — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0941`: The report ends with a one-line summary of total findings per axis and the worst issue within each axis, without ever picking one overall winner across the two axes.

