# provenance — domain-modeling

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-1057`
- `BU-1058`
- `BU-1076`

## Stages

### `01-maintain-glossary-discipline`

- Primary behavior_id: `BU-1059` (`.agents/skills/domain-modeling/SKILL.md (.agents/skills/domain-modeling/SKILL.md L44-46)`)
- Stage-context attachments: `BU-1060`, `BU-1061`, `BU-1062`, `BU-1063`, `BU-1072`, `BU-1073`, `BU-1074`, `BU-1075`, `BU-1077`

### `02-offer-adr`

- Primary behavior_id: `BU-1065` (`.agents/skills/domain-modeling/SKILL.md (.agents/skills/domain-modeling/SKILL.md L68-74)`)
- Stage-context attachments: `BU-1068`, `BU-1069`, `BU-1071`
- Helper attachments: `BU-1066`, `BU-1067`, `BU-1070`

