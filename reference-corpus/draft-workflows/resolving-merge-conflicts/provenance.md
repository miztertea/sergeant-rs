# Provenance — Resolving Merge Conflicts

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W26** `resolving-merge-conflicts`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-045` | resolving-merge-conflicts is a workflow that resolves an in-progress git merge or rebase conflict to completion. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (frontmatter: description) |

## Stages

### `10-research-intent`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-047` | For each conflict, the actor traces the original intent behind each side's change via commit messages, PRs, and issues/tickets before attempting resolution. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 2, line 8) |

Folds the demoted `00-assess-state` checkpoint as a helper (see Adjudication A4 below):

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-046` | The first checkpoint establishes the current merge/rebase state by inspecting git history and the conflicting files. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 1, line 6) |

### `20-resolve-hunks`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-048` | Each conflicting hunk is resolved by preserving both sides' intent where possible, or by picking the side matching the merge's stated goal and recording the trade-off when incompatible; resolution must never invent new behavior, and the merge/rebase must always be carried to completion rather than aborted. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 3, line 10) |

Folds the demoted `30-validate` and `40-finish` checkpoints as helpers, run in sequence (see Adjudication A4 below):

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-049` | After resolving conflicts, the actor discovers and runs the project's automated checks in the order typecheck, then tests, then format, fixing anything the merge broke. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 4, line 12) |
| `BU-P3-050` | The workflow concludes by staging and committing everything, and, if rebasing, continuing until every commit has been rebased. | `reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/SKILL.md` (step 5, line 14) |

## Adjudication A4

N1 adjudication A4 (finding N1-BH-02, `reference-corpus/adjudication-round1.md`): every stage whose `CONTEXT.md` justification was only the §6.5 deterministic-machinery boilerplate is demoted by default, folded into the adjacent judgment-bearing stage as a helper invocation.

- **`00-assess-state` — DEMOTED.** Classified at extraction as deterministic machinery (ladder §6.5) with no "Additional note" checkpoint argument; fails the reimplementation test as an independent checkpoint (inspecting git history/conflicting files is a repeatable operation subordinate to the research checkpoint, not itself a durable state anyone inspects). Folded into `10-research-intent` (its only neighbor, and the stage whose research it precedes) as a helper. `BU-P3-046` survives, re-homed.
- **`30-validate` — DEMOTED.** Same boilerplate-only classification, no surviving checkpoint argument. Its neighbors are `20-resolve-hunks` (before, judgment-bearing) and the also-demoted `40-finish` (after); folded forward into `20-resolve-hunks`, the nearest surviving judgment-bearing stage. `BU-P3-049` survives, re-homed.
- **`40-finish` — DEMOTED.** Same boilerplate-only classification, no surviving checkpoint argument, and no stage after it. Cascades into `20-resolve-hunks` alongside `30-validate` for the same reason. `BU-P3-050` survives, re-homed. `20-resolve-hunks`'s output now carries the `promote` disposition `40-finish`'s output previously carried, since `20-resolve-hunks` is now the workflow's last stage.

