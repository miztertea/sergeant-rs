# 01-list-projects

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the project-listing step is invoked

**Outcome:** only genuine project YAMLs are listed, and an empty result is reported explicitly rather than silently

**Statement (the operative rule):** No single record states this behavior directly; derived from this workflow's own workflow-level helpers (listed below) as a summary of what they collectively establish.

## What must become true here (durable outcome)

Only genuine project YAMLs are listed, and an empty result is reported explicitly rather than silently — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. no `stage`-rung record carries `workflow=list-projects` — only `stage-context`/`helper` members do (a cluster whose checkpoint boundary was never itself classified `stage` rung, per `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` 6.3's own caution). A single stage is inferred so the runtime has a checkpoint to run at all; its content is the workflow-level helpers themselves, and its trigger/outcome are paraphrased from them since no record states a whole-workflow trigger/outcome directly.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0235`: Listing projects enumerates YAML files directly under the Sergeant config directory and skips `config.yaml` specifically, since that file is global config, not a project.
- `BU-0236`: When no project YAMLs are found, the fleet-listing step reports the empty state and exits nonzero with guidance to create a project YAML.

