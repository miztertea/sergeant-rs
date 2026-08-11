# provenance — sync-project-repos

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-sync-existing-repo`

- Primary behavior_id: `BU-0241` (`bin/sgt-sync (bin/sgt-sync L30-39)`)

### `02-clone-missing-repo`

- Primary behavior_id: `BU-0242` (`bin/sgt-sync (bin/sgt-sync L40-48)`)

