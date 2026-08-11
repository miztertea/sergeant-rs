# 01-publish-notification

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the notify step is invoked

**Outcome:** the fleet watcher can discover the update from a durably persisted marker even if the requested transport later fails

**Statement (the operative rule):** Every notify call durably records a metadata-only wake marker (event class and update timestamp) for the task by writing it to a private temp file and atomically renaming it into place, regardless of which notification transport is used.

## What must become true here (durable outcome)

The fleet watcher can discover the update from a durably persisted marker even if the requested transport later fails — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0684`: Unless overridden, the notify step's only externally observable notification side effect is the durable wake marker — no direct injection into any session occurs by default.
- `BU-0685`: In tmux transport mode, if no primary_pane has ever been recorded for the task, the notify step treats the update as satisfied via the durable callback queue when one is registered, and only fails hard if no durable callback origin exists either.
- `BU-0686`: In tmux transport mode, a primary_pane file that exists but is empty is a hard failure to notify — unlike the missing-file case, this is not softened by an available durable callback origin.
- `BU-0687`: In tmux transport mode, if the recorded primary pane's tmux session is no longer running, the notify step treats the update as satisfied via the durable callback queue when one is registered, and only fails hard if no durable callback origin exists either.
- `BU-0688`: In tmux transport mode, when a live primary pane is available, the notify step injects the update message directly into that pane via tmux send-keys.
- `BU-0692`: The notify step's own exit status reflects an earlier durable-callback sync failure even when the requested notification transport itself succeeded.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0689`: An unrecognized SERGEANT_NOTIFY_TRANSPORT value is a hard configuration error.

