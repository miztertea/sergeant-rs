# 01-pre-edit-context-load

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** direct mode is active and an edit is about to be made

**Outcome:** context and the task tracker task state are loaded before any edit

**Statement (the operative rule):** In direct mode, before editing, run the project context-resolution step and the task tracker for the owning task.

## What must become true here (durable outcome)

Context and the task tracker task state are loaded before any edit — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0011`: In direct mode, before editing, reconcile existing workers and preserved worktrees; never duplicate or race work already in progress.

