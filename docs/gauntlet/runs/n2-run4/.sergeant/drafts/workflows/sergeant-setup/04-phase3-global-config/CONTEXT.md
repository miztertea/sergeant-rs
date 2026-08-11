# 04-phase3-global-config

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-phase2-install-checkout/output/outcome.md | L4 | upstream evidence produced by `phase2-install-checkout` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** `~/.config/sergeant/config.yaml` does not exist

**Outcome:** the config file is written only after an explicit confirmed preview

**Statement (the operative rule):** In Phase 3, if `~/.config/sergeant/config.yaml` is missing, the skill asks the user for a `dev_root` path, shows a preview, and asks for confirmation; the file is written only after the user confirms, and the filesystem is left unchanged on any other response.

## What must become true here (durable outcome)

The config file is written only after an explicit confirmed preview — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1279`: If the global config is present and `dev_root` is set, the skill reports `[ok]` without further action.
- `BU-1280`: If the global config is present but invalid YAML, the skill validates it with `yq e '.' ~/.config/sergeant/config.yaml`, reports the parse error, and stops; it must not overwrite the file without a timestamped backup, a diff preview, and explicit confirmation.

