# 05-phase4-new-project-interview

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-phase3-global-config/output/outcome.md | L4 | upstream evidence produced by `phase3-global-config` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a new project YAML is being created

**Outcome:** each question is answered in sequence before the next is asked

**Statement (the operative rule):** Phase 4's interview is for new projects only, and its questions are asked in order, stopping to wait for each answer before proceeding to the next.

## What must become true here (durable outcome)

Each question is answered in sequence before the next is asked — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1281`: If the project YAML already exists and the user wants to modify it, Phase 4 is skipped and Phase 5 (repair existing YAML) is used instead.
- `BU-1283`: The project name, which becomes the YAML filename stem, must match `[a-z0-9_-]+`.
- `BU-1284`: After all interview answers are collected, the skill shows a preview of the complete YAML before writing anything and asks for confirmation.
- `BU-1285`: The file is written only after the user confirms; if the file already exists, a backup is created at `~/.config/sergeant/<name>.yaml.bak.<timestamp>` before writing.

