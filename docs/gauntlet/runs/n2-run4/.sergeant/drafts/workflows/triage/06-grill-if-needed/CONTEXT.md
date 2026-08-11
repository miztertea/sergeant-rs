# 06-grill-if-needed

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** an issue or PR's requirements are underspecified enough to need grilling

**Outcome:** the request is progressively sharpened and decisions are recorded inline rather than left implicit

**Statement (the operative rule):** When a request needs fleshing out, the triage skill runs a structured grilling session — question by question — to sharpen the request's domain terms, recording decisions inline as they land.

## What must become true here (durable outcome)

The request is progressively sharpened and decisions are recorded inline rather than left implicit — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1172`: Everything resolved during a grilling session is captured under the needs-info template's "established so far" section so that work is not lost.

