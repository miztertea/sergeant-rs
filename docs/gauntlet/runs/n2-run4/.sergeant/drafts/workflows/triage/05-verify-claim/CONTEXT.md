# 05-verify-claim

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** an issue or PR reaches the verify step

**Outcome:** the underlying claim (bug report or PR's stated effect) is actually exercised, not just read

**Statement (the operative rule):** Before any grilling, the triage skill verifies the claim holds up: for a bug it reproduces the issue from the reporter's steps, and for a PR it checks out the diff and confirms it does what it claims by running the relevant tests or commands.

## What must become true here (durable outcome)

The underlying claim (bug report or PR's stated effect) is actually exercised, not just read — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1159`: Verification is reported as one of three outcomes — confirmed (with the code path), failed, or insufficient detail — and insufficient detail is treated as a strong signal to move the item to `needs-info` rather than accepted as a clean result.

