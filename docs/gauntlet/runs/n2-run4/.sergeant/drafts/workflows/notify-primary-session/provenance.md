# provenance — notify-primary-session

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0680`

## Stages

### `01-publish-notification`

- Primary behavior_id: `BU-0683` (`bin/sgt-notify (bin/sgt-notify L51-60)`)
- Stage-context attachments: `BU-0684`, `BU-0685`, `BU-0686`, `BU-0687`, `BU-0688`, `BU-0692`
- Helper attachments: `BU-0689`

### `02-capture-wiki-activity`

- Primary behavior_id: `BU-0691` (`bin/sgt-notify (bin/sgt-notify L119-124)`)
- Helper attachments: `BU-0690`

