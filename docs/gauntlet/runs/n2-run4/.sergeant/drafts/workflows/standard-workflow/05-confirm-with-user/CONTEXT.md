# 05-confirm-with-user

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-reconcile-existing-state/output/outcome.md | L4 | upstream evidence produced by `reconcile-existing-state` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** state has been reconciled

**Outcome:** the user is asked only for genuinely unresolved, scope/risk-changing decisions

**Statement (the operative rule):** Step 5 of the standard workflow: ask the user only to confirm unresolved decisions that change scope or risk — repository ownership, user-visible behavior, security/privacy policy, data retention, destructive action, or an irreversible tradeoff that is unknown.

## What must become true here (durable outcome)

The user is asked only for genuinely unresolved, scope/risk-changing decisions — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0030`: The user is not asked to reconfirm an execution mode, plan, or tradeoff already recorded in the conversation or in the task tracker.
- `BU-0281`: If a consequential behavioral seam is undecided, the worker escalates `needs_input` rather than guessing.

