# Installation And Setup

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** Installation, or mise run check/install/update, is invoked.

**Outcome.** Dependencies are verified against their real capability surface, and symlinks/hooks are (re)installed or removed idempotently, before Sergeant is considered usable.

**Completion.** Dependency-check passes for every required dependency.

## How its stages relate

Ordered, trigger-to-outcome:

1. **dependency-check** (`01-dependency-check/`) -- Trigger: the dependency check runs during installation. Outcome: installation does not proceed until both the td-implementation check and the agent-availability check pass.

## Workflow-local helper machinery (not separately packaged)

8 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.

## External shared dependencies (not part of this package)

- **5d td-capability-surface-check** (`BU-0132, BU-0209`) -- dependency-check stage's td verification. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.
