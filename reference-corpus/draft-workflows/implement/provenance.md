# Provenance — Implement

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W23** `implement`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-050` | The implement skill implements a piece of work described by the user against a spec or set of tickets. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (front matter description, lines 3-3) |
| `BU-P2-051` | The implement skill disables automatic model-driven invocation (`disable-model-invocation: true`): it must be explicitly invoked by the user or coordinator, not triggered implicitly by the model recognizing the situation. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (front matter policy, lines 4-4) |
| `BU-P2-052` | The implement workflow should use the tdd workflow where possible, at seams pre-agreed for testing. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 9-9) |
| `BU-P2-054` | Once implementation is done, the code-review skill/workflow is used to review the work. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 13-13) |
| `BU-P3-004` | Cross-harness metadata mirrors the Claude-Code-specific disable-model-invocation flag so a non-Claude-Code harness (OpenCode) enforces the same explicit-invocation-only rule. | `reference/sergeant-upstream/.agents/skills/grill-with-docs/agents/openai.yaml` (policy.allow_implicit_invocation) |

## Stages

### `10-implement-with-tdd`

No directly-cited units (delegated or structural — see the stage's own CONTEXT.md).

### `20-verify`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-053` | During implementation, typechecking and single test files should be run regularly, with the full test suite run once at the end. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 11-11) |

### `30-review`

No directly-cited units (delegated or structural — see the stage's own CONTEXT.md).

### `40-commit`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-055` | The final step of implement is to commit the work to the current branch. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 15-15) |

## Notes

**Synthesis notes:** Explicit-invocation-only (BU-P2-051) — this workflow must never be auto-loaded merely because the task looks like implementation; its cross-harness mirror is BU-P3-004.

