# provenance — dispatch-mode

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0278`

## Stages

### `01-plan-and-decompose`

- Primary behavior_id: `BU-0006` (`AGENTS.md (AGENTS.md L15-20)`)

### `02-dispatch-one-worker-per-repo`

- Primary behavior_id: `BU-0007` (`AGENTS.md (AGENTS.md L15-20)`)

### `03-monitor-and-reconcile`

- Primary behavior_id: `BU-0008` (`AGENTS.md (AGENTS.md L15-20)`)

### `04-validate-harness-selection`

- Primary behavior_id: `BU-0057` (`AGENTS.md (AGENTS.md L186)`)
- Stage-context attachments: `BU-0136`, `BU-0138`, `BU-0139`, `BU-0294`, `BU-0304`, `BU-0321`, `BU-0343`, `BU-0360`, `BU-0879`, `BU-0894`, `BU-0895`
- Helper attachments: `BU-0313`, `BU-0314`, `BU-0315`, `BU-0316`, `BU-0317`

### `05-resolve-and-record-model-pin`

- Primary behavior_id: `BU-0058` (`AGENTS.md (AGENTS.md L187)`)
- Stage-context attachments: `BU-0059`, `BU-0067`, `BU-0068`, `BU-0069`, `BU-0071`, `BU-0072`, `BU-0074`, `BU-0345`, `BU-0346`, `BU-0361`, `BU-0880`, `BU-0881`, `BU-0882`, `BU-0883`, `BU-0884`, `BU-0887`
- Helper attachments: `BU-0066`, `BU-0070`, `BU-0197`, `BU-0885`, `BU-0886`

### `06-bind-and-verify-coordinator-pane`

- Primary behavior_id: `BU-0060` (`AGENTS.md (AGENTS.md L188)`)
- Stage-context attachments: `BU-0075`, `BU-0076`, `BU-0077`, `BU-0287`, `BU-0897`, `BU-0899`, `BU-0900`

### `07-publish-canonical-intent`

- Primary behavior_id: `BU-0135` (`docs/using-sergeant.md (docs/using-sergeant.md L54-58)`)
- Stage-context attachments: `BU-0303`
- Helper attachments: `BU-0323`

### `08-validate-intent-file`

- Primary behavior_id: `BU-0140` (`docs/using-sergeant.md (docs/using-sergeant.md L112-117)`)
- Stage-context attachments: `BU-0327`, `BU-0328`, `BU-0329`, `BU-0330`, `BU-0331`, `BU-0332`
- Helper attachments: `BU-0333`

### `09-prepare-worker-brief`

- Primary behavior_id: `BU-0273` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L36)`)
- Stage-context attachments: `BU-0279`

### `10-reconcile-dispatch-results`

- Primary behavior_id: `BU-0276` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L112)`)
- Stage-context attachments: `BU-0277`

### `11-create-tasks-before-spawn`

- Primary behavior_id: `BU-0284` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L198)`)
- Stage-context attachments: `BU-0285`, `BU-0290`, `BU-0297`, `BU-0298`

### `12-rollback-coordinator-pane-on-abort`

- Primary behavior_id: `BU-0288` (`bin/sgt-dispatch (bin/sgt-dispatch L324-335)`)
- Stage-context attachments: `BU-0296`

### `13-check-drain-admission`

- Primary behavior_id: `BU-0289` (`bin/sgt-dispatch (bin/sgt-dispatch L474-486)`)

### `14-acquire-worktree`

- Primary behavior_id: `BU-0291` (`bin/sgt-dispatch (bin/sgt-dispatch L775-793)`)
- Stage-context attachments: `BU-0292`
- Helper attachments: `BU-0293`, `BU-0925`, `BU-0926`

### `15-handle-spawn-failure`

- Primary behavior_id: `BU-0295` (`bin/sgt-dispatch (bin/sgt-dispatch L916-969)`)

### `16-probe-harness-readiness`

- Primary behavior_id: `BU-0318` (`bin/_sgt-harness.sh (bin/_sgt-harness.sh L204-211)`)
- Stage-context attachments: `BU-0319`, `BU-0320`, `BU-0322`, `BU-0352`, `BU-0353`

### `17-capture-background-session-identity`

- Primary behavior_id: `BU-0362` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L865-878)`)
- Stage-context attachments: `BU-0363`, `BU-0364`

### `18-reattach-after-attach-exit`

- Primary behavior_id: `BU-0365` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L1056-1067)`)
- Stage-context attachments: `BU-0366`

### `19-detect-model-substitution`

- Primary behavior_id: `BU-0367` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L1109-1126)`)
- Stage-context attachments: `BU-0368`

