# install-sergeant — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the dependency check is being run during installation
- **Outcome:** the pull only succeeds when it is a clean fast-forward, never creating a merge commit or silently resolving divergence
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `update-checkout`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-verify-prerequisites` — installation does not proceed until both the td-implementation check and the agent-availability check pass
2. `02-install-symlinks` — every current and future matching script is linked, without needing the installer to be updated per new script
3. `03-uninstall-symlinks` — only hooks this repository actually installed are removed; foreign or already-diverged hooks are preserved
4. `04-update-checkout` — the pull only succeeds when it is a clean fast-forward, never creating a merge commit or silently resolving divergence

