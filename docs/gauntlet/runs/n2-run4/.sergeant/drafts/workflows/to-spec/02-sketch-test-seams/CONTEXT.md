# 02-sketch-test-seams

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-prepare-spec-inputs/output/outcome.md | L4 | upstream evidence produced by `prepare-spec-inputs` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** test seams for the feature are being sketched

**Outcome:** the spec settles on the minimum number of new, high-leverage test seams

**Statement (the operative rule):** Test seams for the spec's feature are sketched preferring existing seams over new ones, at the highest point possible; new seams are proposed only if needed, at the highest point possible, aiming for as few seams as possible (ideally exactly one).

## What must become true here (durable outcome)

The spec settles on the minimum number of new, high-leverage test seams — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0992`: Sketched test seams are checked with the user against their expectations before the spec is written.

