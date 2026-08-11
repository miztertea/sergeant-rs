# Provenance — To Tickets

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W32** `to-tickets`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-058` | Turning a plan, spec, investigation, findings register, PR, or conversation into implementation-ready tracked tickets is a distinct, triggerable procedure with its own dependency-aware breakdown and publication steps. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (frontmatter description, L3-4) |

## Stages

### `00-load-project-context`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-064` | When loading project context for ticket authoring, do not automatically add td instructions to a repository's own guidance files as a side effect. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Load Project Context, L46) |

### `10-extract-decisions-and-unknowns`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-065` | Create a short investigation ticket only when a genuinely blocking unknown cannot be answered from existing evidence, and that ticket must name the exact decision or artifact it is meant to produce. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Extract Decisions and Unknowns, L60-62) |

### `20-confirm-breakdown`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-068` | Unless the user explicitly asked to publish immediately, present the proposed ticket breakdown first and ask only whether granularity, ownership, and blocking edges are correct -- do not re-ask about decisions already made. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Confirm the Breakdown, L100-109) |

### `20-confirm-breakdown` (helper invocation, folded from demoted `30-publish`)

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-070` | Do not mark newly published tasks in_progress; that transition belongs to dispatch or the worker that later starts the work. New tickets remain open until execution actually begins. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Publish to td, L155) |
| `BU-P4-071` | td dependencies are repository-local; for cross-repository blockers, record the counterpart repo/ticket id and exact merge order in both descriptions/logs rather than inventing a native dependency edge td cannot enforce across separate databases. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Publish to td, L149-153) |

### `40-report-frontier`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-072` | When reporting the dispatch frontier, recommend one worker per owning repository as the default concurrency, unless the project explicitly supports more. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Report the Dispatch Frontier, L181-182) |
| `BU-P4-073` | Do not actually dispatch any ticket unless the user asked to begin implementation; reporting the frontier is not itself authorization to start work. | `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Report the Dispatch Frontier, L189) |

## Adjudication A4

- **`30-publish` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `20-confirm-breakdown` as a helper invocation; `BU-P4-070`/`BU-P4-071` move with it. The stage directory is removed; `40-report-frontier`'s Inputs table now points to `20-confirm-breakdown/output/README.md`. No renumbering: `00`, `10`, `20`, `40` remain correctly ordered without `30`.

## Promotion note (`docs/icm/promotion-spec-2026-08-11.md`)

`40-report-frontier`, this package's true (and only) closing stage, declares a `promote` output disposition with no finalize step — one of the 30 of 34 N1 packages in that shape, not one of the 3 (`drain-fleet`, `respond-to-worker`, `to-spec`) that name one. Recorded here per the spec's finalize-gap rule rather than silently promoted; disposition on whether this package needs a finalize step is left to human review at merge time, not applied mechanically by this curation act.

**NEEDS-JUDGMENT resolution (§5):** this package's classification turns on `00-load-project-context`'s `## Delegation` section naming **load-project** as the workflow whose own completion produces this stage's outcome — the smallest instance of the delegation-dependency pattern in the corpus (a single stage, a single named target), but still requiring the target's actual presence in the library for promotion of `to-tickets` to mean anything (§4). Confirmed: `load-project` is already promoted and `status: published` under `.sergeant/workflows/load-project/` (commit `e187c72`, listed in `.sergeant/index.md`). No re-authoring was required — the Delegation target name, the stage's Behavior contract, and its Judgment-required classification carry across byte-for-byte, unedited, per §2's forbidden list. Curation's only act here was verifying the target's presence before promoting the referencing package, not adjudicating anything about either package's content.

