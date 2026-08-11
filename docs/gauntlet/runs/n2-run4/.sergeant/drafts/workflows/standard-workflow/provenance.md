# provenance — standard-workflow

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-load-context`

- Primary behavior_id: `BU-0025` (`AGENTS.md (AGENTS.md L136)`)
- Stage-context attachments: `BU-0134`
- Helper attachments: `BU-0237`, `BU-0238`, `BU-0888`

### `02-load-or-create-task`

- Primary behavior_id: `BU-0026` (`AGENTS.md (AGENTS.md L137)`)

### `03-select-execution-mode`

- Primary behavior_id: `BU-0027` (`AGENTS.md (AGENTS.md L138)`)

### `04-reconcile-existing-state`

- Primary behavior_id: `BU-0028` (`AGENTS.md (AGENTS.md L139)`)

### `05-confirm-with-user`

- Primary behavior_id: `BU-0029` (`AGENTS.md (AGENTS.md L140)`)
- Stage-context attachments: `BU-0030`, `BU-0281`

### `06-execute`

- Primary behavior_id: `BU-0031` (`AGENTS.md (AGENTS.md L141-143)`)

### `07-resolve-blocking-gate`

- Primary behavior_id: `BU-0034` (`AGENTS.md (AGENTS.md L145)`)

### `08-deliver-and-cleanup`

- Primary behavior_id: `BU-0035` (`AGENTS.md (AGENTS.md L146)`)

