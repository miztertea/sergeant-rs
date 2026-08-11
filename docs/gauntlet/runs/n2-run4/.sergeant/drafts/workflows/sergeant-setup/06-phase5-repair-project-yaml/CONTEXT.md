# 06-phase5-repair-project-yaml

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-phase4-new-project-interview/output/outcome.md | L4 | upstream evidence produced by `phase4-new-project-interview` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the existing project YAML fails validation

**Outcome:** the run stops on the parse error rather than attempting further changes

**Statement (the operative rule):** In Phase 5, the skill validates the existing project YAML with `yq e '.' ~/.config/sergeant/<name>.yaml`; if it fails, it reports the parse error and stops without proceeding.

## What must become true here (durable outcome)

The run stops on the parse error rather than attempting further changes — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1287`: The skill computes and displays a minimal diff between the current YAML content and the proposed changes.
- `BU-1288`: The skill asks for confirmation before any write or backup in Phase 5, and only after the user confirms does it create a timestamped backup at `~/.config/sergeant/<name>.yaml.bak.<timestamp>` and then write the new content.
- `BU-1289`: The skill does not create the Phase 5 backup before confirmation, does not apply changes if the user declines, and the backup is mandatory when writing — it is never skipped even if the user asks to skip it.

