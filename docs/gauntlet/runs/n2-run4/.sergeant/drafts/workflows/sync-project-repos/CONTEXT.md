# sync-project-repos — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the sync step runs against an already-cloned repo
- **Outcome:** cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `clone-missing-repo`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-sync-existing-repo` — a diverged or detached-HEAD repo is left untouched with a warning instead of being force-merged
2. `02-clone-missing-repo` — cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on

