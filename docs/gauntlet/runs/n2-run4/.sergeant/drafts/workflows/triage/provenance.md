# provenance — triage

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-1144`
- `BU-1182`
- `BU-1184`
- `BU-1193`

## Stages

### `01-operate-state-machine`

- Primary behavior_id: `BU-1148` (`.agents/skills/triage/SKILL.md (SKILL.md L45)`)
- Stage-context attachments: `BU-1147`, `BU-1149`, `BU-1150`

### `02-surface-attention-queue`

- Primary behavior_id: `BU-1151` (`.agents/skills/triage/SKILL.md (SKILL.md L58-62)`)
- Stage-context attachments: `BU-1152`, `BU-1153`

### `03-gather-context`

- Primary behavior_id: `BU-1154` (`.agents/skills/triage/SKILL.md (SKILL.md L70)`)
- Stage-context attachments: `BU-1155`, `BU-1156`, `BU-1174`, `BU-1187`, `BU-1188`, `BU-1189`, `BU-1190`, `BU-1194`, `BU-1195`

### `04-recommend-and-wait`

- Primary behavior_id: `BU-1157` (`.agents/skills/triage/SKILL.md (SKILL.md L72)`)

### `05-verify-claim`

- Primary behavior_id: `BU-1158` (`.agents/skills/triage/SKILL.md (SKILL.md L74)`)
- Stage-context attachments: `BU-1159`

### `06-grill-if-needed`

- Primary behavior_id: `BU-1160` (`.agents/skills/triage/SKILL.md (SKILL.md L76)`)
- Stage-context attachments: `BU-1172`

### `07-apply-outcome`

- Primary behavior_id: `BU-1161` (`.agents/skills/triage/SKILL.md (SKILL.md L79)`)
- Stage-context attachments: `BU-1162`, `BU-1163`, `BU-1164`, `BU-1165`, `BU-1166`, `BU-1167`, `BU-1173`, `BU-1175`, `BU-1176`, `BU-1177`, `BU-1178`, `BU-1179`, `BU-1180`, `BU-1181`, `BU-1183`, `BU-1185`, `BU-1186`, `BU-1191`, `BU-1192`

### `08-quick-override`

- Primary behavior_id: `BU-1168` (`.agents/skills/triage/SKILL.md (SKILL.md L90)`)
- Stage-context attachments: `BU-1169`, `BU-1170`, `BU-1171`

