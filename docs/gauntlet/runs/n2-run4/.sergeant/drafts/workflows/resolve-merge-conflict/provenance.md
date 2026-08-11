# provenance — resolve-merge-conflict

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-establish-conflict-state`

- Primary behavior_id: `BU-0982` (`.agents/skills/resolving-merge-conflicts/SKILL.md (.agents/skills/resolving-merge-conflicts/SKILL.md L6-6)`)

### `02-resolve-hunk`

- Primary behavior_id: `BU-0984` (`.agents/skills/resolving-merge-conflicts/SKILL.md (.agents/skills/resolving-merge-conflicts/SKILL.md L10-10)`)
- Stage-context attachments: `BU-0983`

### `03-complete-merge`

- Primary behavior_id: `BU-0985` (`.agents/skills/resolving-merge-conflicts/SKILL.md (.agents/skills/resolving-merge-conflicts/SKILL.md L10-10)`)
- Stage-context attachments: `BU-0986`, `BU-0987`

