# provenance — list-projects

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- No record states a whole-workflow trigger/outcome for `list-projects` directly (`.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md` reports "(no stage candidates for this workflow value)"). **Justified design inference:** the workflow-as-a-whole description here is paraphrased from its workflow-level helpers, listed below — not an invented citation.

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0235`
- `BU-0236`

## Stages

### `01-list-projects`

**Justified design inference.** no `stage`-rung record carries `workflow=list-projects` — only `stage-context`/`helper` members do (a cluster whose checkpoint boundary was never itself classified `stage` rung, per `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` 6.3's own caution). A single stage is inferred so the runtime has a checkpoint to run at all; its content is the workflow-level helpers themselves, and its trigger/outcome are paraphrased from them since no record states a whole-workflow trigger/outcome directly.

- Content drawn from workflow-level helpers: `BU-0235`, `BU-0236`

