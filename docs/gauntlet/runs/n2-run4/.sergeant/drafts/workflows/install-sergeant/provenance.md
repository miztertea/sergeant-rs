# provenance — install-sergeant

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-verify-prerequisites`

- Primary behavior_id: `BU-0129` (`docs/getting-started.md (docs/getting-started.md L51-53)`)
- Stage-context attachments: `BU-0132`, `BU-0173`, `BU-0209`, `BU-0210`, `BU-0211`, `BU-0915`, `BU-0916`

### `02-install-symlinks`

- Primary behavior_id: `BU-0204` (`mise.toml (mise.toml L20-23)`)
- Stage-context attachments: `BU-0205`, `BU-0206`

### `03-uninstall-symlinks`

- Primary behavior_id: `BU-0207` (`mise.toml (mise.toml L93-104)`)
- Stage-context attachments: `BU-0208`

### `04-update-checkout`

- Primary behavior_id: `BU-0213` (`mise.toml (mise.toml L292)`)

