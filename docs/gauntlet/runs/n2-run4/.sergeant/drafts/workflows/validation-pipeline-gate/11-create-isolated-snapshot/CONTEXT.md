# 11-create-isolated-snapshot

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the project-validation step creates the isolated validation snapshot

**Outcome:** the code actually validated is provably the exact reviewed commit, never a snapshot that silently drifted during creation

**Statement (the operative rule):** The isolated validation code snapshot is created as a --shared --no-checkout clone of the source worktree's root, then hard-checked-out to exactly the reviewed HEAD; the launch fails outright if the resulting HEAD or tree cleanliness does not match what was reviewed.

## What must become true here (durable outcome)

The code actually validated is provably the exact reviewed commit, never a snapshot that silently drifted during creation — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0381`: A validation checkout owner token (combining the lock purpose, lock identity, and the reviewed HEAD) is written into the isolated snapshot's own .git directory immediately after creation, binding that specific checkout to this specific launch.
- `BU-0392`: Immediately before running the validation pipeline, the validation worker verifies the isolated snapshot's current HEAD still matches the expected reviewed HEAD and that the snapshot is still validation-clean; either mismatch is a fatal error.

