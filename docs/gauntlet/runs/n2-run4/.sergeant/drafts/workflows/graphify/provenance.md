# provenance — graphify

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-run-graph-generation`

- Primary behavior_id: `BU-0133` (`docs/getting-started.md (docs/getting-started.md L161)`)
- Stage-context attachments: `BU-0184`, `BU-0198`, `BU-0199`, `BU-0245`, `BU-0248`, `BU-0250`, `BU-0251`, `BU-0252`, `BU-0254`, `BU-0262`, `BU-0263`, `BU-0264`
- Helper attachments: `BU-0196`, `BU-0246`, `BU-0247`, `BU-0249`

### `02-recover-from-failed-publish`

- Primary behavior_id: `BU-0253` (`bin/sgt-graphify (bin/sgt-graphify L207-238)`)

