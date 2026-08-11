# Project Registration

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A project needs its context resolved or its registration validated.

**Outcome.** Context loading is confirmed complete via an observable evidence artifact, never merely by having run the command.

**Completion.** Confirm-context-loaded.

## How its stages relate

Ordered, trigger-to-outcome:

1. **confirm-context-loaded** (`01-confirm-context-loaded/`) -- Trigger: project context loading is claimed complete. Outcome: completeness is defined by an observable evidence artifact, not merely having run the command.

## Unattached stage-context evidence, not materialized

5 `stage-context` behavior_id(s), across 4 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Workflow-local helper machinery (not separately packaged)

11 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.

## External shared dependencies (not part of this package)

- **5b dev-root-relative-path-resolution** (`BU-0051, BU-0193`) -- resolves repo paths from project YAML. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.
- **5c project-name-identity** (`BU-0052, BU-0195`) -- derives addressable project name. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.
