# provenance — dag-run

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0201`
- `BU-0579`
- `BU-0580`
- `BU-0865`
- `BU-0866`

## Stages

### `01-resolve-stage-brief`

- Primary behavior_id: `BU-0202` (`schema/project.yaml.example (schema/project.yaml.example L112-115)`)
- Stage-context attachments: `BU-0867`, `BU-0868`

### `02-advance-on-dependency-completion`

- Primary behavior_id: `BU-0203` (`schema/project.yaml.example (schema/project.yaml.example L117-120)`)
- Stage-context attachments: `BU-0869`

### `03-verify-dag-prerequisites`

- Primary behavior_id: `BU-0859` (`bin/sgt-dag-run (bin/sgt-dag-run L39-42)`)
- Stage-context attachments: `BU-0860`, `BU-0861`, `BU-0862`, `BU-0863`, `BU-0864`, `BU-0870`

### `04-run-dispatch-hook`

- Primary behavior_id: `BU-0871` (`bin/sgt-dag-dispatch-hook (bin/sgt-dag-dispatch-hook L21)`)
- Stage-context attachments: `BU-0872`, `BU-0873`, `BU-0874`, `BU-0875`, `BU-0876`

