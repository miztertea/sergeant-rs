# Provenance — Resolving Merge Conflicts

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W26** `resolving-merge-conflicts`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-045` | resolving-merge-conflicts is a workflow that resolves an in-progress git merge or rebase conflict to completion. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (frontmatter: description) |

## Stages

### `00-assess-state`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-046` | The first checkpoint establishes the current merge/rebase state by inspecting git history and the conflicting files. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 1, line 6) |

### `10-research-intent`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-047` | For each conflict, the actor traces the original intent behind each side's change via commit messages, PRs, and issues/tickets before attempting resolution. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 2, line 8) |

### `20-resolve-hunks`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-048` | Each conflicting hunk is resolved by preserving both sides' intent where possible, or by picking the side matching the merge's stated goal and recording the trade-off when incompatible; resolution must never invent new behavior, and the merge/rebase must always be carried to completion rather than aborted. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 3, line 10) |

### `30-validate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-049` | After resolving conflicts, the actor discovers and runs the project's automated checks in the order typecheck, then tests, then format, fixing anything the merge broke. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 4, line 12) |

### `40-finish`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-050` | The workflow concludes by staging and committing everything, and, if rebasing, continuing until every commit has been rebased. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 5, line 14) |

