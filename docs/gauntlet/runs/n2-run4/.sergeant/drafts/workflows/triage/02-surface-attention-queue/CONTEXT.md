# 02-surface-attention-queue

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the maintainer asks what needs triage attention

**Outcome:** three ordered buckets of items are presented, oldest first within each

**Statement (the operative rule):** When showing what needs attention, the triage skill presents three buckets — unlabeled items, items in `needs-triage`, and `needs-info` items with reporter activity since the last triage notes — ordered oldest first.

## What must become true here (durable outcome)

Three ordered buckets of items are presented, oldest first within each — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1152`: When pull requests are in scope for triage, the 'what needs attention' discovery buckets surface only external PRs — a collaborator's own in-flight PR is not included as triage work.
- `BU-1153`: The external-PR-only discovery filter only limits what is surfaced automatically — a PR explicitly named by the maintainer is always triaged regardless of who authored it.

