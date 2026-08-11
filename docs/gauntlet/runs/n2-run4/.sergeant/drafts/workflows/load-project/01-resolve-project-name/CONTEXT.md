# 01-resolve-project-name

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the project name for a task is not already known exactly

**Outcome:** an exact registered name is confirmed before context loading proceeds

**Statement (the operative rule):** If a project name is unknown, the fleet-listing step is run and an exact registered name is required before proceeding, rather than guessing or fuzzy-matching a project.

## What must become true here (durable outcome)

An exact registered name is confirmed before context loading proceeds — per the Statement above, which is the operative rule this stage exists to enforce.

