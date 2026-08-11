# 01-run-grill-with-docs

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the grill-with-docs skill is invoked

**Outcome:** the resulting interview both stress-tests the plan and leaves behind ADR/glossary docs

**Statement (the operative rule):** Invoking grill-with-docs runs a /grilling session using the /domain-modeling skill, so the interview it drives also produces ADR and glossary documentation as it goes.

## What must become true here (durable outcome)

The resulting interview both stress-tests the plan and leaves behind ADR/glossary docs — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. single-behavior workflow candidate (invoke-grill-with-docs) with no downstream `stage`/`stage-context`/`helper` record — the sole behavior `BU-0969` is materialized as this one stage so the runtime has a checkpoint to run at all; the workflow and the stage are co-extensive by construction, not two independently evidenced facts.

