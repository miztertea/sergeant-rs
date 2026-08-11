# 09-verify-intent-consistency

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the project-validation step checks whether the canonical intent revision matches before launching

**Outcome:** any divergence between the three intent copies, or between a recorded revision and the file's real hash, blocks validation rather than validating a possibly-stale or inconsistent intent

**Statement (the operative rule):** A coordinator-owned validation run only proceeds if the fleet-level, repo-state-level, and worktree-level copies of .sergeant-intent.md are byte-identical to each other AND their recorded revision hashes agree AND that revision re-verifies against the fleet copy's actual current content.

## What must become true here (durable outcome)

Any divergence between the three intent copies, or between a recorded revision and the file's real hash, blocks validation rather than validating a possibly-stale or inconsistent intent — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0372`: Before proceeding with any validation launch, the canonical intent revision recorded at the fleet, repo-state, and worktree levels must all match (via _sgt_intent_revision_matches); a mismatch is fatal, requiring an audited human decision or a new revision rather than proceeding on stale or divergent intent.
- `BU-0388`: The validation worker refuses to proceed unless the canonical validation intent's own current revision hash exactly matches the revision it was invoked with, before doing anything else.
- `BU-0393`: Immediately before invoking the validation pipeline, the validation worker re-computes the canonical intent's revision hash and fails (recording exited:2) if it no longer matches the expected revision, catching a content change made after the initial startup check rather than validating against it.

