# Provenance — Grilling

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W28** `grilling`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-005` | grilling is a workflow that interviews the user to stress-test a plan, decision, or idea, triggered by an explicit request or grill trigger phrases. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (frontmatter: description) |

## Stages

### `00-interview-loop`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-006` | The interview proceeds systematically down a decision tree, resolving dependent decisions in order, with the actor offering a recommended answer alongside each question. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 6) |
| `BU-P3-007` | Within the interview loop, only one question is posed at a time, and the actor waits for the user's answer before asking the next. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 8) |
| `BU-P3-008` | The interview loop draws a firm line: facts discoverable by exploring the environment must be looked up by the actor; only genuine decisions are put to the user, and the actor waits for the user's answer on each. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 10) |

### `10-confirm-understanding`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-009` | The workflow may not proceed to action until the user explicitly confirms shared understanding has been reached. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 12) |

