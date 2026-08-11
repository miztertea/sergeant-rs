# 02-clone-missing-repo

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-sync-existing-repo/output/outcome.md | L4 | upstream evidence produced by `sync-existing-repo` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a configured repo path is missing

**Outcome:** cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on

**Statement (the operative rule):** The sync step clones a repo only when its path does not exist and a `url` is configured; a path that exists but is not a git repo, or has no configured url, is skipped with a warning rather than being overwritten or guessed at.

## What must become true here (durable outcome)

Cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on — per the Statement above, which is the operative rule this stage exists to enforce.

