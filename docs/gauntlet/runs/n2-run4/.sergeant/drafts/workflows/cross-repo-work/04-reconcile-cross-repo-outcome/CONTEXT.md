# 04-reconcile-cross-repo-outcome

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-handoff-plan/output/outcome.md | L4 | upstream evidence produced by `handoff-plan` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a cross-repo outcome is being reported

**Outcome:** completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset

**Statement (the operative rule):** A cross-repo outcome is never reported complete until every owning repository has reached a terminal result or has an explicit preserved blocker.

## What must become true here (durable outcome)

Completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset — per the Statement above, which is the operative rule this stage exists to enforce.

