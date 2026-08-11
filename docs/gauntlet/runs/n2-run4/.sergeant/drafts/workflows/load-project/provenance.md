# provenance — load-project

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-resolve-project-name`

- Primary behavior_id: `BU-0255` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L17-19)`)

### `02-load-repo-context`

- Primary behavior_id: `BU-0257` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L28-29)`)
- Stage-context attachments: `BU-0256`, `BU-0258`

### `03-edit-and-validate-project`

- Primary behavior_id: `BU-0260` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L45-46)`)
- Stage-context attachments: `BU-0261`

