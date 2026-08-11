# provenance — callback-protocol

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0761`
- `BU-0763`
- `BU-0765`
- `BU-0768`
- `BU-0815`

## Stages

### `01-resolve-callback-executable`

- Primary behavior_id: `BU-0214` (`docs/callbacks.md (docs/callbacks.md L10-15)`)
- Stage-context attachments: `BU-0215`, `BU-0762`, `BU-0764`, `BU-0766`

### `02-register-origin`

- Primary behavior_id: `BU-0216` (`docs/callbacks.md (docs/callbacks.md L31-33)`)
- Stage-context attachments: `BU-0217`, `BU-0218`, `BU-0767`, `BU-0781`, `BU-0782`

### `03-sync-and-produce-events`

- Primary behavior_id: `BU-0219` (`docs/callbacks.md (docs/callbacks.md L67-70)`)
- Stage-context attachments: `BU-0229`, `BU-0604`, `BU-0681`, `BU-0682`, `BU-0783`, `BU-0785`, `BU-0786`, `BU-0787`, `BU-0788`, `BU-0789`
- Helper attachments: `BU-0784`

### `04-enqueue-event`

- Primary behavior_id: `BU-0220` (`docs/callbacks.md (docs/callbacks.md L80-82)`)
- Stage-context attachments: `BU-0221`, `BU-0769`, `BU-0770`, `BU-0771`, `BU-0772`, `BU-0773`, `BU-0774`, `BU-0775`, `BU-0778`, `BU-0780`, `BU-0813`
- Helper attachments: `BU-0814`

### `05-invoke-consumer`

- Primary behavior_id: `BU-0222` (`docs/callbacks.md (docs/callbacks.md L96-98)`)
- Stage-context attachments: `BU-0223`, `BU-0224`, `BU-0234`, `BU-0790`

### `06-process-acknowledgement`

- Primary behavior_id: `BU-0225` (`docs/callbacks.md (docs/callbacks.md L134-138)`)
- Stage-context attachments: `BU-0226`, `BU-0776`, `BU-0777`, `BU-0791`, `BU-0792`, `BU-0793`, `BU-0794`, `BU-0795`, `BU-0796`, `BU-0803`

### `07-retry-delivery`

- Primary behavior_id: `BU-0227` (`docs/callbacks.md (docs/callbacks.md L146-149)`)
- Stage-context attachments: `BU-0228`, `BU-0797`, `BU-0798`, `BU-0799`, `BU-0800`, `BU-0801`, `BU-0802`, `BU-0804`, `BU-0807`, `BU-0808`, `BU-0809`

