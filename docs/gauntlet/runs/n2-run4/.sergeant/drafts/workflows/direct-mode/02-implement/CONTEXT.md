# 02-implement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-pre-edit-context-load/output/outcome.md | L4 | upstream evidence produced by `pre-edit-context-load` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** direct mode is active

**Outcome:** the owning task tracker task is claimed/created and implementation proceeds test-first

**Statement (the operative rule):** In direct mode, claim or create the owning task tracker task, then implement TDD-first in the requested checkout or an isolated worktree.

## What must become true here (durable outcome)

The owning task tracker task is claimed/created and implementation proceeds test-first — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0013`: In direct mode, the default branch is never edited; a feature branch is created or reused before the first implementation change.

