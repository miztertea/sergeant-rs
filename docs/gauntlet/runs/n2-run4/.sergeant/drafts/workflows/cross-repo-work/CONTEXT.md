# cross-repo-work — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a requested outcome is being decomposed across repositories
- **Outcome:** completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `reconcile-cross-repo-outcome`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-assign-ownership` — ownership assignment is unambiguous (exactly one owner per behavior) and scoped to repos that actually must change
2. `02-order-dependencies` — dispatch does not proceed with a cyclic dependency graph; the cycle is broken by design instead
3. `03-handoff-plan` — the outcome (plan-only vs implement) matches exactly what was requested, and multi-repo direct editing by the primary session never happens
4. `04-reconcile-cross-repo-outcome` — completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset

