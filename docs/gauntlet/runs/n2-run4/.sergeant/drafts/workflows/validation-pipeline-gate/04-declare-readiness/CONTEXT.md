# 04-declare-readiness

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** native validation and independent reviews all pass

**Outcome:** readiness is durably recorded with intent/head/review evidence before the coordinator is notified, and the worker itself never invokes the validation pipeline

**Statement (the operative rule):** After native validation and independent reviews report zero blockers, the worker writes `.sergeant-validation-ready` with the recorded `intent_revision`, current `head_sha`, and passed values for `standards_review`, `spec_review`, and `readiness_review`, then notifies the coordinator; the worker must not run the validation pipeline.

## What must become true here (durable outcome)

Readiness is durably recorded with intent/head/review evidence before the coordinator is notified, and the worker itself never invokes the validation pipeline — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0309`: Validation requires a clean worktree at the committed HEAD; readiness evidence is never created from an uncommitted diff, and the branch is committed before readiness is published.
- `BU-0373`: Validation only launches if all three review axes recorded on the validation-ready marker (standards, spec, readiness) are exactly "passed"; any other value fails with a message naming the specific axis and its actual recorded value.
- `BU-0374`: The worker's code tree must be clean (per _sgt_worktree_is_validation_clean) before a validation snapshot is taken; a dirty tree fails validation launch outright.
- `BU-0914`: A worktree is considered 'validation clean' only if it has no staged or unstaged diffs against HEAD and no untracked files other than Sergeant's own .sergeant-* control files.

