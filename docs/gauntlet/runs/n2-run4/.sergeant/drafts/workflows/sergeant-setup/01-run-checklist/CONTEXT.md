# 01-run-checklist

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a checklist step is reached

**Outcome:** already-satisfied steps are skipped silently and every step's outcome is recorded visibly

**Statement (the operative rule):** The skill maintains a visible, numbered checklist in terminal output; before each step it verifies whether the step is already complete and skips it without prompting if so, and after each step it writes an `[ok]` or `[skipped]` status line.

## What must become true here (durable outcome)

Already-satisfied steps are skipped silently and every step's outcome is recorded visibly — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1267`: When a phase fails, the skill stops the current run with actionable output identifying the last completed phase.
- `BU-1268`: On the next invocation the checklist starts over from Phase 1 but skips every phase that already passes verification; resumability works by re-checking each phase before acting on it, not by persisting state between runs.

