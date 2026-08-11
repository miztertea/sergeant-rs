# Skill Adoption

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** An external skill is being adopted.

**Outcome.** The six-step vetting procedure (read SKILL.md and referenced scripts; confirm source/update mechanism; check filesystem/shell/network/Git/credential actions; verify no conflict with AGENTS.md/safety policy; pin/lock the source where supported; test in a disposable repo/worktree) is completed before broad installation.

**Completion.** All six checks done.

## Zero materialized stages

No `representation: stage` record in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` carries this candidate's `workflow` value. Per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` and `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3, this is not resolved by inventing a stage: this package has no `NN-*/` directories and `workflow.toml` declares `stages = []`. See `provenance.md` for the evidence this candidate boundary rests on instead.

## Workflow-local helper machinery (not separately packaged)

1 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.
