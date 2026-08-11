# 04-apply-fix

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-hypothesize-and-test/output/outcome.md | L4 | upstream evidence produced by `hypothesize-and-test` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the root cause is understood and a fix is about to be written

**Outcome:** a test-first fix is preferred, conditioned on a correct seam being available

**Statement (the operative rule):** The regression test is written before the fix, but only if a correct seam for it actually exists.

## What must become true here (durable outcome)

A test-first fix is preferred, conditioned on a correct seam being available — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0964`: A correct seam is one that exercises the real bug pattern as it occurs at the call site; a seam that is too shallow (e.g. a single-caller test for a bug that needs multiple callers) gives false confidence rather than real coverage.
- `BU-0965`: If no candidate seam for a regression test is correct, that absence is itself recorded as a finding — the codebase's architecture is preventing the bug from being locked down — and it is flagged for the post-mortem phase.
- `BU-0966`: When a correct seam exists, the fix is applied in order: turn the minimised repro into a failing test at that seam, watch it fail, apply the fix, watch it pass, then re-run the original (un-minimised) Phase 1 feedback loop against the full scenario.

