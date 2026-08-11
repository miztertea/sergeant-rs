# 24-stop-validation-pane

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** _stop_validation_pane is called for a repo with a recorded validation_pane

**Outcome:** a validation pane with incomplete ownership provenance is never terminated on an assumption

**Statement (the operative rule):** The fleet cleanup step refuses to stop a recorded validation pane whose PID, process-group, and start-time provenance are not all recorded together, dying rather than guessing which process to signal.

## What must become true here (durable outcome)

A validation pane with incomplete ownership provenance is never terminated on an assumption — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0622`: Before terminating a live validation pane, the fleet cleanup step re-verifies the pane's live identity against the recorded validation-pane identity and dies immediately on any mismatch, rather than terminating a pane that may no longer be the validation worker.
- `BU-0623`: Before terminating a validation checkout's process group, the fleet cleanup step verifies the recorded owning PID's start time still matches what was recorded, refusing (dying) if the PID appears to have been reused.
- `BU-0662`: The fleet cleanup step validates a recorded validation checkout's ownership provenance before any destructive step (pane kills, worktree removal) runs for that repo, so that rejecting an invalid validation checkout leaves the live worker pane and fleet evidence completely unchanged.
- `BU-0663`: The fleet cleanup step recognizes exactly two shapes of validation-checkout ownership provenance, a full four-field 'exact' record or a legacy single-head record with none of the four fields, and treats any other combination of present and absent provenance fields as invalid rather than guessing which shape was intended.
- `BU-0664`: Verifying a validation checkout's identity requires its own git-common-dir to carry a sergeant-validation-owner file containing the exact expected owner string, in addition to matching path identity, git-dir identity, and HEAD; path or content equivalence alone is not sufficient to prove ownership.

