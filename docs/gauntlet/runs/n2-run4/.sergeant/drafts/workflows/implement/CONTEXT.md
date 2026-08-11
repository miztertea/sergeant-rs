# implement — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** implementation work has pre-agreed seams
- **Outcome:** the work is durably recorded in version control
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `commit-implementation`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-implement-at-seams` — implementation at those seams follows the TDD discipline rather than an ad hoc approach
2. `02-review-implementation` — the finished work receives a code review before being considered done
3. `03-commit-implementation` — the work is durably recorded in version control

