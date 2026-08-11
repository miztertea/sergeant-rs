# 13-publish-worker-readiness-handshake

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the validation worker is ready to publish its readiness handshake

**Outcome:** the readiness handshake cannot be published, or later replayed by an unrelated process, without matching this exact revision+pane+pid+start-time tuple

**Statement (the operative rule):** The validation worker requires a live TMUX_PANE and a resolvable process start time for itself before publishing its own readiness handshake, and requires the coordinator to still be alive at that moment; the handshake value binds together the expected intent revision, the pane, the child PID, and the child's process start time.

## What must become true here (durable outcome)

The readiness handshake cannot be published, or later replayed by an unrelated process, without matching this exact revision+pane+pid+start-time tuple — per the Statement above, which is the operative rule this stage exists to enforce.

