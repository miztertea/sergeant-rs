# 02-reproduce-and-minimize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-build-feedback-loop/output/outcome.md | L4 | upstream evidence produced by `build-feedback-loop` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the feedback loop from Phase 1 has been run and gone red

**Outcome:** minimisation only begins once the loop is confirmed to be catching the right bug, reliably, with the symptom on record

**Statement (the operative rule):** Before minimising, Phase 2 confirms three things about the red loop: it reproduces the user's exact described failure (not a different nearby one, since the wrong bug means the wrong fix), it reproduces across multiple runs (or at a high enough rate for non-deterministic bugs), and the exact symptom is captured so later phases can verify the fix addresses it.

## What must become true here (durable outcome)

Minimisation only begins once the loop is confirmed to be catching the right bug, reliably, with the symptom on record — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0952`: Minimising a reproduced bug cuts inputs, callers, config, data, and steps one at a time, re-running the loop after every cut, keeping only what is load-bearing for the failure.
- `BU-0953`: Minimisation is done once every remaining element is load-bearing — removing any single one of them would make the loop go green.
- `BU-0954`: Hypothesising is not started until both reproduction and minimisation are complete.

