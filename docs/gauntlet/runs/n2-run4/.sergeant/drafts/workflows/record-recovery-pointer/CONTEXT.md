# record-recovery-pointer — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the task-tracker memory step is invoked with a worktree path
- **Outcome:** git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `bind-worktree-identity`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-bind-worktree-identity` — git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test

## Cross-cutting mechanics

This workflow's only stage carries these directly in its own `CONTEXT.md` (too few stages for `_config/` to mean "shared across more than one stage" per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`).

