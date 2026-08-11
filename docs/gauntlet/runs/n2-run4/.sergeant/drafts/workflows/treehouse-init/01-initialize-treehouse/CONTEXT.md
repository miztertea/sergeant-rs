# 01-initialize-treehouse

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** treehouse initialization is run for a repo

**Outcome:** initialization is idempotent: an already-initialized repo is reported as such rather than re-initialized

**Statement (the operative rule):** No single record states this behavior directly; derived from this workflow's own workflow-level helpers (listed below) as a summary of what they collectively establish.

## What must become true here (durable outcome)

Initialization is idempotent: an already-initialized repo is reported as such rather than re-initialized — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. no `stage`-rung record carries `workflow=treehouse-init` — only `stage-context`/`helper` members do (a cluster whose checkpoint boundary was never itself classified `stage` rung, per `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` 6.3's own caution). A single stage is inferred so the runtime has a checkpoint to run at all; its content is the workflow-level helpers themselves, and its trigger/outcome are paraphrased from them since no record states a whole-workflow trigger/outcome directly.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0299`: Treehouse initialization is idempotent per repo: a repo that already has `treehouse.toml` is reported as already initialized rather than being re-initialized.

