# 01-sync-existing-repo

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the sync step runs against an already-cloned repo

**Outcome:** a diverged or detached-HEAD repo is left untouched with a warning instead of being force-merged

**Statement (the operative rule):** For an already-cloned repo, the sync step pulls with `--ff-only`, and skips the pull with a warning on detached HEAD or a failed (diverged/no-upstream) fast-forward, rather than forcing a merge or losing local state.

## What must become true here (durable outcome)

A diverged or detached-HEAD repo is left untouched with a warning instead of being force-merged — per the Statement above, which is the operative rule this stage exists to enforce.

