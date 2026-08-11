# 07-phase6-verify-installation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../06-phase5-repair-project-yaml/output/outcome.md | L4 | upstream evidence produced by `phase5-repair-project-yaml` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the project YAML has just been written

**Outcome:** verification proceeds strictly in order and halts at the first failure

**Statement (the operative rule):** In Phase 6, after the YAML is written the skill runs the fleet-listing step, the project context-resolution step, the status step, and the sync step in order, reporting the result of each, stopping and reporting the first failure with its full output and not continuing to the next command until the previous one succeeds.

## What must become true here (durable outcome)

Verification proceeds strictly in order and halts at the first failure — per the Statement above, which is the operative rule this stage exists to enforce.

