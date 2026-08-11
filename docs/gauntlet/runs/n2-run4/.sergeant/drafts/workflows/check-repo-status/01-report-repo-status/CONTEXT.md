# 01-report-repo-status

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the status step is invoked for one or more repos

**Outcome:** each repo's status is reported from only verified, currently-observable git/filesystem state, never assumed

**Statement (the operative rule):** No single record states this behavior directly; derived from this workflow's own workflow-level helpers (listed below) as a summary of what they collectively establish.

## What must become true here (durable outcome)

Each repo's status is reported from only verified, currently-observable git/filesystem state, never assumed — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. no `stage`-rung record carries `workflow=check-repo-status` — only `stage-context`/`helper` members do (a cluster whose checkpoint boundary was never itself classified `stage` rung, per `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` 6.3's own caution). A single stage is inferred so the runtime has a checkpoint to run at all; its content is the workflow-level helpers themselves, and its trigger/outcome are paraphrased from them since no record states a whole-workflow trigger/outcome directly.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0239`: For each repo, the status step reports NOT CLONED or NOT A GIT REPO and skips further git inspection for that repo, rather than attempting git commands against a missing or non-repo path.
- `BU-0240`: When a repo has an upstream branch, the status step reports the ahead/behind commit counts relative to that upstream whenever either is nonzero.

