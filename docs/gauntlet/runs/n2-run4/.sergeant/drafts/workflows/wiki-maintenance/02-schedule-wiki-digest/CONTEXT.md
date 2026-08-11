# 02-schedule-wiki-digest

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-operate-wiki-digest/output/outcome.md | L4 | upstream evidence produced by `operate-wiki-digest` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a coordinator has installed a scheduled job for the wiki-digest job

**Outcome:** a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution

**Statement (the operative rule):** Scheduling the daily digest is not reported complete until the job definition, executable path, environment, last exit status, and generated page have all been verified.

## What must become true here (durable outcome)

A scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution — per the Statement above, which is the operative rule this stage exists to enforce.

