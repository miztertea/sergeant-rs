# 03-validate-and-review

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-implement/output/outcome.md | L4 | upstream evidence produced by `implement` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** direct mode implementation has reached validation/review/shipping

**Outcome:** direct-mode work passes through the same validation/review/gate steps as dispatched work

**Statement (the operative rule):** In direct mode, repository-native validation, independent reviews, and the final shipping gate are run exactly as a dispatched worker would run them.

## What must become true here (durable outcome)

Direct-mode work passes through the same validation/review/gate steps as dispatched work — per the Statement above, which is the operative rule this stage exists to enforce.

