# load-project — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the project name for a task is not already known exactly
- **Outcome:** the edit is validated against resolved context output, not just YAML syntax validity
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `edit-and-validate-project`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-resolve-project-name` — an exact registered name is confirmed before context loading proceeds
2. `02-load-repo-context` — sync is deferred until actually needed, and a sync failure halts rather than proceeding with a missing repo
3. `03-edit-and-validate-project` — the edit is validated against resolved context output, not just YAML syntax validity

