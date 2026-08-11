# 04-update-checkout

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-uninstall-symlinks/output/outcome.md | L4 | upstream evidence produced by `uninstall-symlinks` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the toolchain/task runner run update is invoked

**Outcome:** the pull only succeeds when it is a clean fast-forward, never creating a merge commit or silently resolving divergence

**Statement (the operative rule):** Updating pulls the latest changes with `git pull --ff-only` rather than an ordinary pull, then reinstalls symlinks.

## What must become true here (durable outcome)

The pull only succeeds when it is a clean fast-forward, never creating a merge commit or silently resolving divergence — per the Statement above, which is the operative rule this stage exists to enforce.

