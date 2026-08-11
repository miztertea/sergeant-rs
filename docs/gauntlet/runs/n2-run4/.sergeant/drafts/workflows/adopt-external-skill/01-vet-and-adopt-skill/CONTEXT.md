# 01-vet-and-adopt-skill

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** an external skill is being adopted

**Outcome:** the six-step vetting procedure is completed before broad installation

**Statement (the operative rule):** Before adopting an external skill: read its complete SKILL.md and referenced scripts, confirm its source and update mechanism, check its filesystem/shell/network/Git/credential actions, verify no conflict with `AGENTS.md` or safety policy, pin or lock its source where supported, and test it in a disposable repository or worktree before broad installation.

## What must become true here (durable outcome)

The six-step vetting procedure is completed before broad installation — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. single-behavior workflow candidate (adopt-external-skill) with no downstream `stage`/`stage-context`/`helper` record — the sole behavior `BU-0119` is materialized as this one stage so the runtime has a checkpoint to run at all; the workflow and the stage are co-extensive by construction, not two independently evidenced facts.

