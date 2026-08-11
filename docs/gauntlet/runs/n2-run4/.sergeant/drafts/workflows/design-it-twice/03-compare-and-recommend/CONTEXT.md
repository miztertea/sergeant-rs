# 03-compare-and-recommend

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-spawn-design-subagents/output/outcome.md | L4 | upstream evidence produced by `spawn-design-subagents` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** multiple alternative interface designs have been produced and are ready for review

**Outcome:** the user receives a structured, sequential presentation and a comparison along three named axes

**Statement (the operative rule):** The resulting alternative designs are presented to the user sequentially and compared in prose by depth (leverage at the interface), locality (where change concentrates), and seam placement.

## What must become true here (durable outcome)

The user receives a structured, sequential presentation and a comparison along three named axes — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1056`: After comparing the alternative designs, an explicit, opinionated recommendation is given for which design is strongest and why, with a hybrid proposed if elements from different designs would combine well — never left as an unopinionated menu of options.

