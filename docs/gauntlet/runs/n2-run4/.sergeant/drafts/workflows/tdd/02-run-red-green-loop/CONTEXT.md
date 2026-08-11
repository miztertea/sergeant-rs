# 02-run-red-green-loop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-agree-seams/output/outcome.md | L4 | upstream evidence produced by `agree-seams` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** TDD work is being sequenced across multiple tests and their implementations

**Outcome:** work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases

**Statement (the operative rule):** Horizontal slicing — writing all tests first, then all implementation — is an anti-pattern because bulk tests verify imagined shape rather than user-facing behavior and commit to test structure before the implementation is understood; vertical slices (one test, then one minimal implementation, repeated as a tracer bullet) are used instead.

## What must become true here (durable outcome)

Work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1134`: In the red-green loop, the failing test is written first and only enough code is written to pass it, without anticipating future tests or adding speculative features.
- `BU-1135`: Each TDD cycle covers exactly one seam, one test, and one minimal implementation.
- `BU-1136`: Refactoring is not part of the red-green loop; it belongs to the separate review stage.

