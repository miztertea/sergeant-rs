# 05-declare-bug-fixed

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-apply-fix/output/outcome.md | L4 | upstream evidence produced by `apply-fix` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a fix has been applied and passed its regression test

**Outcome:** the bug is not declared done until all five completion conditions hold

**Statement (the operative rule):** Phase 6 requires five things before declaring a bug fixed: the original repro no longer reproduces, the regression test passes (or seam absence is documented), all tagged debug instrumentation is removed, any throwaway prototypes are deleted or clearly relocated, and the hypothesis that turned out correct is recorded in the commit/PR message.

## What must become true here (durable outcome)

The bug is not declared done until all five completion conditions hold — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0968`: After a bug is fixed, the actor asks what would have prevented it; if the answer is architectural (no good test seam, tangled callers, hidden coupling) it hands off to the /improve-codebase-architecture skill with the specifics, and this recommendation is made only after the fix is in, not before.

