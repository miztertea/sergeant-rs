# provenance — code-review

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-run-parallel-axis-reviews`

- Primary behavior_id: `BU-0928` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L6-11)`)
- Stage-context attachments: `BU-0933`, `BU-0934`, `BU-0935`, `BU-0936`, `BU-0937`, `BU-0938`

### `02-prepare-review-inputs`

- Primary behavior_id: `BU-0931` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L23-23)`)
- Stage-context attachments: `BU-0929`, `BU-0930`, `BU-0932`, `BU-0939`

### `03-aggregate-review-report`

- Primary behavior_id: `BU-0940` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L78-78)`)
- Stage-context attachments: `BU-0941`

