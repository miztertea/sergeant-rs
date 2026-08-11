# provenance — invoke-grill-with-docs

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Established entirely by `BU-0969` (single-behavior workflow candidate — no `stage`/`stage-context`/`helper` record in this corpus carries `workflow=invoke-grill-with-docs`).

## Stages

### `01-run-grill-with-docs`

**Justified design inference.** single-behavior workflow candidate (invoke-grill-with-docs) with no downstream `stage`/`stage-context`/`helper` record — the sole behavior `BU-0969` is materialized as this one stage so the runtime has a checkpoint to run at all; the workflow and the stage are co-extensive by construction, not two independently evidenced facts.

- Sole source behavior: `BU-0969`

