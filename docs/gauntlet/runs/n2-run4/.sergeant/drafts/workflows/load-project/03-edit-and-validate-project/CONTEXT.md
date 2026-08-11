# 03-edit-and-validate-project

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-load-repo-context/output/outcome.md | L4 | upstream evidence produced by `load-repo-context` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a project YAML has been edited

**Outcome:** the edit is validated against resolved context output, not just YAML syntax validity

**Statement (the operative rule):** After editing a project, context-resolution step is run and every edited field needed by agents is required to appear in the resolved output before the edit is considered validated.

## What must become true here (durable outcome)

The edit is validated against resolved context output, not just YAML syntax validity — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0261`: If project registration/edit validation fails, the prior YAML is restored or the new file is left uncommitted, and the exact command error is reported.

