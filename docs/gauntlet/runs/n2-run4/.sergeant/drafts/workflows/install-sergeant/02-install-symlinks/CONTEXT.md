# 02-install-symlinks

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-verify-prerequisites/output/outcome.md | L4 | upstream evidence produced by `verify-prerequisites` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the install task runs

**Outcome:** every current and future matching script is linked, without needing the installer to be updated per new script

**Statement (the operative rule):** Installation links every `sgt-*`, `_sgt-*.sh`, and the wiki-digest job script by globbing the bin directory rather than enumerating filenames by name, because enumerating by name silently broke the install whenever a new `bin/_sgt-*.sh` helper was added.

## What must become true here (durable outcome)

Every current and future matching script is linked, without needing the installer to be updated per new script — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0205`: Install removes legacy `oc-inject` links (a deleted feature) from `~/.local/bin` and `~/.config/opencode/plugins/`, but only when the target is a symlink (`-L` check), never an ordinary file.
- `BU-0206`: Install links git hooks from `scripts/hooks/` into `.git/hooks/`, but only when both the hooks source and destination directories exist.

