# 03-complete-merge

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-resolve-hunk/output/outcome.md | L4 | upstream evidence produced by `resolve-hunk` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a merge/rebase has conflicts

**Outcome:** the merge/rebase always reaches a resolved state rather than being abandoned mid-way

**Statement (the operative rule):** A merge or rebase conflict is always resolved rather than abandoned — `--abort` is never used.

## What must become true here (durable outcome)

The merge/rebase always reaches a resolved state rather than being abandoned mid-way — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0986`: After resolving conflicting hunks, the project's automated checks are discovered and run (typically typecheck, then tests, then format), and anything the merge broke is fixed.
- `BU-0987`: Once checks pass, everything is staged and committed to finish the merge; if rebasing, the rebase is continued until all commits are rebased.

