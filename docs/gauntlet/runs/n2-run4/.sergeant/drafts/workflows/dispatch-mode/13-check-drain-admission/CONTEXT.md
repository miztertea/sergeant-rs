# 13-check-drain-admission

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** dispatch is about to create its first durable side effect

**Outcome:** a race between a concurrent drain and this dispatch's admission is closed by holding the lock across the critical window, and any ambiguous drain record blocks rather than admits

**Statement (the operative rule):** The drain admission lock is acquired and held through the task tracker task creation (the first side effect), so a concurrent drain either waits until admission is committed or wins the lock first and blocks this dispatch; malformed, empty, or expired drain records fail closed and block dispatch.

## What must become true here (durable outcome)

A race between a concurrent drain and this dispatch's admission is closed by holding the lock across the critical window, and any ambiguous drain record blocks rather than admits — per the Statement above, which is the operative rule this stage exists to enforce.

