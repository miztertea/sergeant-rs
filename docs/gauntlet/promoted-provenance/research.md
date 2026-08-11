# Provenance — Research

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W27** `research`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-040` | research is a workflow that investigates a question against high-trust primary sources and writes the findings to a Markdown file in the repository. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (frontmatter: description) |
| `BU-P3-041` | The research workflow is delegated to a background/asynchronous execution context so the requester's foreground work is not blocked while sources are read. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (body line 6) |

## Stages

### `00-investigate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-042` | Research must be conducted against primary sources (official docs, source code, specs, first-party APIs) rather than secondary summaries, with every claim traced back to its owning source. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 1, line 10) |
| `BU-P3-043` (helper invocation, folded from demoted `10-write-findings`) | The investigation's output is a single Markdown file where every claim carries a source citation. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 2, line 11) |
| `BU-P3-044` (helper invocation, folded from demoted `10-write-findings`) | The findings file is placed according to the repository's existing note-keeping convention, or in a sensible location (with the choice explicitly stated) if no convention exists. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 3, line 12) |

## Notes

**Synthesis notes:** Delegated to a background execution context (BU-P3-041) — that delegation is a *scheduling* property of how research is invoked, not a stage of the procedure itself.

## Adjudication A4

- **`10-write-findings` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `00-investigate` as a helper invocation; `BU-P3-043`/`BU-P3-044` and the stage's citations move with it. The stage directory is removed; `00-investigate` absorbs the workflow's terminal `promote` output disposition.

## Promotion note (docs/icm/promotion-spec-2026-08-11.md §1)

This package declares a `promote` output disposition (`00-investigate/output/README.md`) with no finalize step at its closing (and only) stage — one of the 30 of 34 N1 packages in that shape, not one of the 3 (`drain-fleet`, `respond-to-worker`, `to-spec`) that name one. Recorded here per the spec's finalize-gap rule rather than silently promoted; disposition on whether this package needs a finalize step is left to human review at merge time, not applied mechanically by this curation act.

