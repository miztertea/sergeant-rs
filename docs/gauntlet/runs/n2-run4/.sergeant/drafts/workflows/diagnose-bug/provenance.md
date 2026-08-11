# provenance — diagnose-bug

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

## Stages

### `01-build-feedback-loop`

- Primary behavior_id: `BU-0944` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L14-14)`)
- Stage-context attachments: `BU-0943`, `BU-0945`, `BU-0946`, `BU-0947`, `BU-0948`, `BU-0949`, `BU-0950`

### `02-reproduce-and-minimize`

- Primary behavior_id: `BU-0951` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L66-70)`)
- Stage-context attachments: `BU-0952`, `BU-0953`, `BU-0954`

### `03-hypothesize-and-test`

- Primary behavior_id: `BU-0955` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L84-84)`)
- Stage-context attachments: `BU-0956`, `BU-0957`, `BU-0958`, `BU-0959`, `BU-0960`, `BU-0961`, `BU-0962`

### `04-apply-fix`

- Primary behavior_id: `BU-0963` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L110-110)`)
- Stage-context attachments: `BU-0964`, `BU-0965`, `BU-0966`

### `05-declare-bug-fixed`

- Primary behavior_id: `BU-0967` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L126-132)`)
- Stage-context attachments: `BU-0968`

