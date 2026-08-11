# 03-uninstall-symlinks

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-install-symlinks/output/outcome.md | L4 | upstream evidence produced by `install-symlinks` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** uninstall:hooks runs

**Outcome:** only hooks this repository actually installed are removed; foreign or already-diverged hooks are preserved

**Statement (the operative rule):** Uninstalling git hooks removes a hook symlink only when it is a symlink whose target still matches this repository's own `scripts/hooks/<name>` file, leaving any other hook (or a hook that no longer points here) untouched.

## What must become true here (durable outcome)

Only hooks this repository actually installed are removed; foreign or already-diverged hooks are preserved — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0208`: Uninstalling command symlinks removes a `~/.local/bin` entry only when it is a symlink and its resolved target path contains `/sergeant/bin/`, so a same-named file that is not actually a link back into this repository is never removed.

