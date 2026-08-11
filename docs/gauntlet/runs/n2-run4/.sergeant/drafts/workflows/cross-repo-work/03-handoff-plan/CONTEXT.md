# 03-handoff-plan

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-order-dependencies/output/outcome.md | L4 | upstream evidence produced by `order-dependencies` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** cross-repo decomposition is complete

**Outcome:** the outcome (plan-only vs implement) matches exactly what was requested, and multi-repo direct editing by the primary session never happens

**Statement (the operative rule):** If the user requested planning only, the procedure stops after returning repository briefs, acceptance evidence, and the dependency graph, without dispatching or editing any repository; when implementation was requested, `dispatch` is loaded and the primary session never edits several repositories directly.

## What must become true here (durable outcome)

The outcome (plan-only vs implement) matches exactly what was requested, and multi-repo direct editing by the primary session never happens — per the Statement above, which is the operative rule this stage exists to enforce.

