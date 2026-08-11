# Provenance — Code Review

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W24** `code-review`.

## Stages

### `00-pin-fixed-point`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-004` | The review's fixed comparison point is whatever the user specified (commit SHA, branch, tag, HEAD~N, etc); if the user did not specify one, the actor must ask for it before proceeding. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 1: Pin the fixed point, lines 19-19) |
| `BU-P2-006` | Before spawning the two parallel review sub-agents, the actor must confirm the fixed point resolves (`git rev-parse`) and the diff is non-empty; a bad ref or empty diff must fail at this point, not inside the sub-agents. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 1: Pin the fixed point, lines 23-23) |

### `10-identify-spec-source`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-007` | The spec source for the Spec axis is located in a fixed priority order: issue references in commit messages, then a path the user passed as an argument, then a PRD/spec file under docs/, specs/, or .scratch/ matching the branch or feature name, then — if nothing is found — asking the user; if the user says no spec exists, the Spec sub-agent is skipped and reports 'no spec available'. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 2: Identify the spec source, lines 27-32) |

### `20-parallel-review-standards`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-001` | The Standards review axis asks whether the code conforms to the repository's documented coding standards. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (front matter / Process intro, lines 8-8) |
| `BU-P2-002` | The Spec review axis asks whether the code faithfully implements the originating issue, PRD, or spec. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (front matter / Process intro, lines 9-9) |
| `BU-P2-009` | A documented repository standard always overrides the smell baseline: where the repo's own standard endorses something the baseline would flag, the smell is suppressed. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 3: Identify the standards sources, lines 40-40) |
| `BU-P2-010` | Every baseline smell is a labelled judgment-call heuristic, never a hard violation, and the reviewer must skip anything tooling already enforces. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 3: Identify the standards sources, lines 41-41) |

### `30-parallel-review-spec`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-003` | The Standards and Spec reviews run as parallel, isolated sub-agents so neither review's context pollutes the other, and this skill aggregates both sets of findings afterward. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Process intro, lines 11-11) |
| `BU-P2-013` | The Standards sub-agent's prompt must include the diff command and commit list, the located standards-source files plus the full smell baseline pasted in (since the sub-agent has no other access to it), and a brief asking it to report per-file/hunk hard standard violations (cited to the standard) and baseline smells (named and quoted), distinguishing hard violations from judgment calls, skipping tooling-enforced items, under 400 words. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 4: Standards sub-agent prompt, lines 62-66) |
| `BU-P2-014` | The Spec sub-agent's prompt must include the diff command and commit list, the path or fetched contents of the spec, and a brief asking it to report missing/partial requirements, scope creep (unasked-for behavior), and requirements that look implemented but wrong, quoting the spec line for each finding, under 400 words. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 4: Spec sub-agent prompt, lines 68-72) |
| `BU-P2-015` | If no spec is found, the Spec sub-agent is skipped entirely and the final report notes this explicitly. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 4: spec missing handling, lines 74-74) |

### `40-aggregate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-016` | The two sub-agent reports are presented under separate `## Standards` and `## Spec` headings, verbatim or lightly cleaned, and must never be merged or reranked against each other since the two axes are deliberately kept separate. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 5: Aggregate, lines 78-78) |
| `BU-P2-017` | The aggregated report ends with a one-line summary of total findings per axis and the worst issue within each axis, without picking one overall winner across axes. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 5: Aggregate, lines 80-80) |
| `BU-P2-018` | The two-axis design exists because a change can pass one axis and fail the other (standards-compliant but spec-wrong, or spec-correct but convention-breaking), and reporting them separately stops either axis from masking the other's failure. | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Why two axes, lines 84-87) |

**Curation note (promotion, 2026-08-11):** `40-aggregate` is this workflow's
true closing stage and declares a `promote` output (its `output/README.md`)
with no finalize step — one of the 30 corpus packages in that shape per
`docs/icm/promotion-spec-2026-08-11.md` §1's D9 observation. Not a
promotion blocker (D9 is an open question, not a numbered rule); recorded
here rather than silently laundered, disposition left to human review.

## Notes

**Synthesis notes:** The two-axis separation is the durable design point (BU-P2-018), not the sub-agent mechanism that happens to isolate the two reviews from each other.

