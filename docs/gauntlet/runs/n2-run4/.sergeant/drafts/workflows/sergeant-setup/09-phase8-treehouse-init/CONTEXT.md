# 09-phase8-treehouse-init

## Inputs

| File | Layer | Why |
|---|---|---|
| ../08-phase7-task-tracker-init/output/outcome.md | L4 | upstream evidence produced by `phase7-task-tracker-init` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the treehouse session manager may or may not be installed

**Outcome:** Treehouse is initialized only with consent, and its absence or decline never blocks overall setup completion

**Statement (the operative rule):** In Phase 8, if the treehouse session manager is present on `PATH` the skill offers to initialize Treehouse worktree pools, running the treehouse-init step only on confirmation, skipping silently on decline or absence, and never marking setup incomplete because Treehouse was skipped.

## What must become true here (durable outcome)

Treehouse is initialized only with consent, and its absence or decline never blocks overall setup completion — per the Statement above, which is the operative rule this stage exists to enforce.

