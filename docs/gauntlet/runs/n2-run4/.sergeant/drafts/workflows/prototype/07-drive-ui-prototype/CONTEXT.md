# 07-drive-ui-prototype

## Inputs

| File | Layer | Why |
|---|---|---|
| ../06-build-ui-prototype/output/outcome.md | L4 | upstream evidence produced by `build-ui-prototype` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a UI prototype is handed over to the user

**Outcome:** the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise

**Statement (the operative rule):** Once a UI prototype is handed over, its URL (and variant keys) are surfaced to the user, who flips between variants at their own pace; feedback that mixes elements across variants (e.g. wanting one part from one variant and another part from a different one) is treated as the actual design signal.

## What must become true here (durable outcome)

The user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise — per the Statement above, which is the operative rule this stage exists to enforce.

