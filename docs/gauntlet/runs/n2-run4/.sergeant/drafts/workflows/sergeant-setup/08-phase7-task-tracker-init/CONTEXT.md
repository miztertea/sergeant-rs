# 08-phase7-task-tracker-init

## Inputs

| File | Layer | Why |
|---|---|---|
| ../07-phase6-verify-installation/output/outcome.md | L4 | upstream evidence produced by `phase6-verify-installation` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a project repository's task tracker status is unknown

**Outcome:** each repository's task tracker state is either confirmed `[ok]` or consent-gated before initialization

**Statement (the operative rule):** In Phase 7, for each project repository the skill checks the task tracker; if initialized it reports `[ok]`, and if not it shows the task tracker command and asks for consent, running it only after the user confirms and reporting a decline as `[skipped]` in the Phase 9 summary while continuing.

## What must become true here (durable outcome)

Each repository's task tracker state is either confirmed `[ok]` or consent-gated before initialization — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1292`: The skill does not initialize the task tracker in any repository that was not registered in the current project YAML.

