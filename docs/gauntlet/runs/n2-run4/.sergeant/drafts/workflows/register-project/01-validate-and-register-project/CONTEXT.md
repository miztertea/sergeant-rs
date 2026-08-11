# 01-validate-and-register-project

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a new project YAML is being registered

**Outcome:** the project file satisfies all six named field-shape requirements

**Statement (the operative rule):** Registering a project requires the project file's `name` field to match its filename, every repository to have a unique name and correct path, clone URLs present for repositories the sync step may clone, roles/groups identifying ownership, agent instructions containing commands and observable constraints rather than vague quality slogans, and `graphify.output` (when used) to be one project-level path outside source repos.

## What must become true here (durable outcome)

The project file satisfies all six named field-shape requirements — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. single-behavior workflow candidate (register-project) with no downstream `stage`/`stage-context`/`helper` record — the sole behavior `BU-0131` is materialized as this one stage so the runtime has a checkpoint to run at all; the workflow and the stage are co-extensive by construction, not two independently evidenced facts.

