# Provenance — Grill with Docs

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W29** `grill-with-docs`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-002` | grill-with-docs must be invoked explicitly by the user or another procedure; the assistant must not decide on its own to start it. | `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (frontmatter: disable-model-invocation) |

## Stages

### `00-interview-loop`

No directly-cited units (delegated or structural — see the stage's own CONTEXT.md).

### `10-confirm-understanding`

No directly-cited units against the confirmation-gate content. The following are cited as a helper invocation folded from the demoted `20-capture-decisions` stage:

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-001` | grill-with-docs is a workflow that sharpens a plan or design through interview while producing durable design docs (ADRs, glossary) as a side effect. | `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (frontmatter: description) |
| `BU-P3-003` | grill-with-docs is defined by composing two other procedures: it runs the grilling interview loop while using the domain-modeling skill to capture ADRs/glossary entries as decisions land. | `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (body line 7) |

## Notes

**Synthesis notes:** This is the corpus's cleanest example of workflow composition **without** nesting — representable today by inlining `grilling`'s two stages ahead of the capture step, which is why it does *not* raise an engine gap. Explicit-invocation-only (BU-P3-002).

## Adjudication A4

- **`20-capture-decisions` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `10-confirm-understanding` as a helper invocation; `BU-P3-001`/`BU-P3-003` and the stage's citations move with it. The stage directory is removed; `10-confirm-understanding` absorbs the workflow's terminal `promote` output disposition.

