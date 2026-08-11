# provenance — review-findings-routing

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-route-finding`

- Primary behavior_id: `BU-0096` (`README.md (README.md L318)`)
- Stage-context attachments: `BU-0097`, `BU-0101`, `BU-0311`, `BU-0726`, `BU-0729`, `BU-0737`, `BU-0738`, `BU-0745`, `BU-0747`, `BU-0748`, `BU-0749`, `BU-0750`, `BU-0751`, `BU-0753`, `BU-0755`, `BU-0760`
- Helper attachments: `BU-0098`, `BU-0099`, `BU-0100`, `BU-0102`, `BU-0302`, `BU-0711`, `BU-0720`, `BU-0722`, `BU-0723`, `BU-0724`, `BU-0725`, `BU-0727`, `BU-0728`, `BU-0730`, `BU-0731`, `BU-0732`, `BU-0733`, `BU-0734`, `BU-0744`, `BU-0746`, `BU-0752`, `BU-0754`, `BU-0756`

### `02-preserve-retry-evidence-on-failure`

- Primary behavior_id: `BU-0103` (`README.md (README.md L324-328)`)
- Stage-context attachments: `BU-0104`, `BU-0312`, `BU-0716`, `BU-0717`, `BU-0718`, `BU-0719`, `BU-0735`, `BU-0736`, `BU-0739`, `BU-0740`, `BU-0741`, `BU-0742`, `BU-0743`

### `03-publish-blocked-gate`

- Primary behavior_id: `BU-0714` (`bin/sgt-review-findings (bin/sgt-review-findings L97-106)`)
- Stage-context attachments: `BU-0715`, `BU-0721`, `BU-0757`, `BU-0758`, `BU-0759`
- Helper attachments: `BU-0712`, `BU-0713`

