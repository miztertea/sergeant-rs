# provenance — to-tickets

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-load-ticket-context`

- Primary behavior_id: `BU-1306` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L32)`)
- Stage-context attachments: `BU-1307`, `BU-1308`, `BU-1309`, `BU-1310`

### `02-draft-tickets`

- Primary behavior_id: `BU-1316` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L80-84)`)
- Stage-context attachments: `BU-1312`, `BU-1313`, `BU-1314`, `BU-1315`, `BU-1317`, `BU-1318`, `BU-1319`, `BU-1333`

### `03-review-breakdown`

- Primary behavior_id: `BU-1320` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L100-101)`)
- Stage-context attachments: `BU-1321`

### `04-publish-tickets`

- Primary behavior_id: `BU-1322` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L114-115)`)
- Stage-context attachments: `BU-1323`, `BU-1324`, `BU-1325`, `BU-1326`, `BU-1327`

### `05-validate-published-graph`

- Primary behavior_id: `BU-1328` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L166)`)
- Stage-context attachments: `BU-1329`

### `06-report-dispatch-frontier`

- Primary behavior_id: `BU-1330` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L179-180)`)
- Stage-context attachments: `BU-1331`, `BU-1332`

