# 07-resolve-blocking-gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../06-execute/output/outcome.md | L4 | upstream evidence produced by `execute` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a worker reaches needs_input, blocked, or an ask-user gate

**Outcome:** only genuinely missing decisions are solicited, recorded in the task tracker, and remediation continues without redundant re-asks

**Statement (the operative rule):** Step 8 of the standard workflow: for `needs_input`, `blocked`, or ask-user gates, read the exact finding, obtain only genuinely missing user decisions, record them in the task tracker, and continue approved remediation without asking again merely to dispatch.

## What must become true here (durable outcome)

Only genuinely missing decisions are solicited, recorded in the task tracker, and remediation continues without redundant re-asks — per the Statement above, which is the operative rule this stage exists to enforce.

