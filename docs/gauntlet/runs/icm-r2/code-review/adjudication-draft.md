# Package adjudication: code-review

Producer pass, ICM-R2 nine-package pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 8-9). Method: `reference/proposal-icm-r-procedure-authority.md` §8.
Record shape: `docs/icm/record-shapes.md` §6. This is a draft for independent
review, not a landed change (ADR decision 6, promotable-only review).

## Original intention

Two-axis review of a diff — **Standards** (does the code conform to this
repo's documented coding standards, plus a fixed Fowler-smell baseline that
applies even when the repo documents nothing?) and **Spec** (does the code
faithfully implement the originating issue/PRD/spec?) — run as two isolated
sub-agents so neither review's context pollutes the other, then aggregated
side by side without merging or reranking. Source: `reference/sergeant-
upstream/.agents/skills/code-review/SKILL.md`, N1 candidate **W24**
(`docs/gauntlet/contracts/N1.md`, `reference-corpus/synthesis.md` §1).

## Current trigger and outcome

Trigger: a diff needs review before merge (invoked directly, or delegated
from `worker-mission`/`implement`). Outcome: an aggregated report with
`## Standards` and `## Spec` sections, each produced by an isolated
sub-agent review, never merged or reranked against each other, closing with
a one-line per-axis summary (`.sergeant/workflows/code-review/CONTEXT.md`,
`40-aggregate/CONTEXT.md`).

## Driver and admission boundary

Driver: stage-actor throughout — five ordinary actor stages, no Captain
dialogue and no deterministic/execute stage
(`.sergeant/workflows/code-review/workflow.toml`). Admission boundary:
in-work — the workflow receives an already-admitted diff/Work; it does not
itself decide whether Work should exist (PL-4, §5.6: "given an already-
defined intent... can Sergeant execute this procedure durably from
admission to a terminal result").

## Behavior-unit dispositions

Every unit below is cross-checked against three independent sources: the
promoted package tree (`.sergeant/workflows/code-review/`), the promotion's
own provenance record (`docs/gauntlet/promoted-provenance/code-review.md`),
and the earlier reference-corpus classification that fed it
(`reference-corpus/behavior-units/P2.ndjson`, `reference-corpus/helper-map.md`,
`reference-corpus/shared-context-map.md`, `reference-corpus/synthesis.md`).
Four units the corpus's own synthesis already classified and destined for
this package (BU-P2-005, BU-P2-008, BU-P2-011, BU-P2-012) have no file
anywhere under `.sergeant/workflows/code-review/` and no
`classification-ledger.md` entry rejecting or parking them — they were
silently dropped between classification and promotion, not adjudicated out.

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-P2-004` | SKILL.md Step 1, L19 | PL-5 | J0 — ask user if fixed point unspecified | STAND | `00-pin-fixed-point/CONTEXT.md` |
| `BU-P2-005` | SKILL.md Step 1, L21 | PL-6 (workflow-local helper) | J1 — deterministic git mechanics | HARVEST | `scripts/capture-diff.sh`, referenced from `00-pin-fixed-point/CONTEXT.md`; classified `helper`/`workflow-local` in `reference-corpus/helper-map.md` line 332, never materialized |
| `BU-P2-006` | SKILL.md Step 1, L23 | PL-5 | J2 — confirm fixed point resolves and diff is non-empty | STAND | `00-pin-fixed-point/CONTEXT.md` |
| `BU-P2-007` | SKILL.md Step 2, L27-32 | PL-5 | J2 (priority walk) → J0 (ask if none found) | STAND | `10-identify-spec-source/CONTEXT.md` |
| `BU-P2-001` | SKILL.md front matter, L8 | PL-4 (workflow-level axis definition) | n/a — descriptive | HARVEST | package `CONTEXT.md` Purpose; currently a stage-20 "must-do" bullet, but it defines the workflow's Standards axis at large, not a stage-20-specific action |
| `BU-P2-002` | SKILL.md front matter, L9 | PL-4 | n/a | HARVEST | package `CONTEXT.md` Purpose; currently stranded inside stage 20 (Standards) even though it defines the *Spec* axis — misfiled by axis |
| `BU-P2-008` | SKILL.md Step 3, L38 | PL-3 (workflow-local shared context) | J5 — always-present fallback content, applies even when the repo documents nothing | HARVEST | `references/smell-baseline.md`; classified `shared-context`/`workflow-local` in `reference-corpus/shared-context-map.md` Part 3 line 343, never materialized |
| `BU-P2-009` | SKILL.md Step 3, L40 | PL-5 | J5 — documented repo standard always overrides the baseline | FOLD | package `CONTEXT.md` Authority envelope (governing constraint), referenced again from the merged review stage |
| `BU-P2-010` | SKILL.md Step 3, L41 | PL-5 | J2 — label as judgment call, skip tooling-enforced items | STAND | merged `20-30-parallel-review/CONTEXT.md` |
| `BU-P2-011` | SKILL.md Step 3, L45-56 | PL-3 (workflow-local shared-helper content — the 12-smell list itself) | n/a — reference content | HARVEST | `references/smell-baseline.md`; classified `shared-helper`/`workflow-local` in `reference-corpus/helper-map.md` line 332, never materialized. Without this the Standards sub-agent prompt required by `BU-P2-013` cannot actually be assembled — the promoted package instructs pasting in "the full smell baseline" that exists nowhere in the package. |
| `BU-P2-003` | SKILL.md Process intro, L11 | PL-5 (one stage-execution shape: a single message, two concurrent isolated sub-agent calls) | J5 — must not run sequentially or pollute the other axis's context | HARVEST | merged `20-30-parallel-review/CONTEXT.md`; currently represented as two *sequential* engine stages (`20-parallel-review-standards` → `30-parallel-review-spec`), with `30`'s Inputs table naming `20`'s output as an L4 dependency — this cannot represent "single message, two calls" concurrency and reintroduces the cross-axis ordering the source explicitly forbids |
| `BU-P2-012` | SKILL.md Step 4, L60 | PL-6 (workflow-local helper — invocation mechanics) | J1 | HARVEST | merged `20-30-parallel-review/CONTEXT.md`; the reference corpus's own classification record (`reference-corpus/behavior-units/P2.ndjson`) already names this unit's stage `20-30-parallel-review` — a single merged stage. The promoted package split it into two stages instead, contradicting its own already-adjudicated classification. |
| `BU-P2-013` | SKILL.md Step 4, L62-66 | PL-5/PL-6 | J2 — assemble the Standards sub-agent prompt | HARVEST | merged `20-30-parallel-review/CONTEXT.md`; currently stranded inside stage `30`, whose own declared purpose is "an isolated review against the identified spec source" — stage `20`'s actor (whose job is the Standards review) has no operative instructions for spawning the sub-agent it's supposed to isolate |
| `BU-P2-014` | SKILL.md Step 4, L68-72 | PL-5/PL-6 | J2 — assemble the Spec sub-agent prompt | HARVEST | merged `20-30-parallel-review/CONTEXT.md`; correctly filed under stage 30 today, folds in unchanged |
| `BU-P2-015` | SKILL.md Step 4, L74 | PL-5 | J4 — consumes stage 10's recorded "no spec" decision | HARVEST | merged `20-30-parallel-review/CONTEXT.md`; correctly filed under stage 30 today, folds in unchanged |
| `BU-P2-016` | SKILL.md Step 5, L78 | PL-5 | J5 — never merge or rerank the two axes | STAND | `40-aggregate/CONTEXT.md` |
| `BU-P2-017` | SKILL.md Step 5, L80 | PL-5 | J2 — write the closing per-axis summary | STAND | `40-aggregate/CONTEXT.md` |
| `BU-P2-018` | SKILL.md "Why two axes", L84-87 | PL-4 (design rationale) | n/a | STAND | `40-aggregate/CONTEXT.md` Notes + package `CONTEXT.md` Notes for reviewers (both already present, correctly) |

Structural findings not tied to one behavior unit:

1. **No `## Authority envelope`** in the package `CONTEXT.md`, required by
   `docs/icm/convention.md` §6.1 and proposal §7.2 for every workflow
   Layer-1 file.
2. **No `## Bounded judgment` section on any of the five stages.** Every
   stage instead carries a generic "## Judgment required" paragraph
   ("this is an actor stage... inspect evidence, choose among
   alternatives...") that is identical, word-for-word, across all five
   stages. This is exactly the "identical generic reasons copied across
   ...rungs" pattern the ladder treats as evidence the ladder was not
   actually applied (proposal §5.9, R11) — it names no J2 delegation, no J1
   local-choice boundary, and no J0 trigger, so it does not meet ADR
   decision 4's "always present, even when it is only 'inherits workflow
   envelope unchanged'" bar; it is not that content, just a paraphrase of
   the ladder's own preamble.
3. **Broken self-reference.** The package `CONTEXT.md` says "See
   `provenance.md` for the complete stage-to-behavior-unit mapping" — no
   `provenance.md` exists anywhere under
   `.sergeant/workflows/code-review/`. The real file is
   `docs/gauntlet/promoted-provenance/code-review.md`, which `index.md`
   correctly names but `CONTEXT.md` does not.
4. **Review independence (convention.md §6.3).** Not a violation found
   in-package: `code-review` is itself a review workflow for *external*
   diffs; nothing in its five stages reviews or promotes this package's own
   output. The independence question that actually applies is at the
   ICM-R2 pilot level — this record is produced by a producer position and
   requires a separate reviewer execution with no edit authority before
   Captain's reconcile-and-publish pass, which this record's own "Review
   and promotion policy" section below states explicitly.

## Surviving package design

Four actor stages (down from five, by merging the two that jointly
implement one concurrent-dispatch checkpoint), all `driver: stage-actor`:

1. `00-pin-fixed-point` — resolve/validate the fixed comparison point
   (unchanged in stage count; gains a `scripts/capture-diff.sh` reference
   and a real `## Bounded judgment` section).
2. `10-identify-spec-source` — fixed priority order, ending in asking the
   user (unchanged; gains `## Bounded judgment`).
3. `20-30-parallel-review` — **new**, merges the former
   `20-parallel-review-standards` and `30-parallel-review-spec`. One actor
   turn spawns both isolated sub-agents in a single message
   (`BU-P2-003`/`BU-P2-012`), matching the reference corpus's own
   classification record which already names this stage
   `20-30-parallel-review`. Reads `references/smell-baseline.md` for the
   Standards sub-agent's pasted-in content.
4. `40-aggregate` — unchanged in role; gains `## Bounded judgment`.

Package `CONTEXT.md` gains `## Authority envelope`; the stray front-matter
axis definitions (`BU-P2-001`/`BU-P2-002`) move into `## Purpose`; the
`provenance.md` pointer is corrected to the real path. New workflow-local
content: `references/smell-baseline.md` (the twelve-smell list plus the
"repo overrides" and "always a judgment call" rules, `BU-P2-008`/`009`/
`010`/`011`) and `scripts/capture-diff.sh` (`BU-P2-005`).

## Inputs and outputs

Inputs across the run: the user's stated fixed point (or their answer when
asked), the repo's documented coding-standard files (if any), the
workflow-local smell baseline (always), and the identified spec source (or
an explicit "no spec" answer). Outputs: `00`/`10`/`20-30` each produce
`evidence`-disposition Layer-4 artifacts (Work-branch record of how the
outcome was reached, not merged by default); `40-aggregate` produces the
single `promote`-disposition artifact — the two-axis report — that is this
workflow's actual deliverable.

## Review and promotion policy

Artifact class: a `.sergeant/workflows/` procedure package (this
reconciliation's own output) plus, downstream, the per-run review report it
produces for its callers. Draft location: this record, plus
`docs/gauntlet/runs/icm-r2/code-review/draft/`, mirroring the live
package's structure. Independent reviewer: a fresh ICM-R2 execution with
explicit inputs (this record, the draft tree, the sources cited above) and
no authority to edit either — satisfying `docs/icm/convention.md` §6.3's
review-independence test rather than assuming a shared workflow wrapper is
enough. Acceptance criteria: every behavior unit above is dispositioned
with a resolving citation (this record's own self-check, §8.10); the
reviewer additionally challenges rung order, the Captain/workflow boundary
(n/a here — no Captain stage), the stage/helper boundary on the new merged
stage, and whether the two newly-harvested workflow-local artifacts are
genuinely single-consumer (§8.11). Promotion action: Captain's
reconcile-and-publish pass (§8.12) replaces the live
`.sergeant/workflows/code-review/` tree with the accepted draft in one
change, preserving `docs/gauntlet/promoted-provenance/code-review.md`'s
provenance trail and updating it for the newly-harvested units. Failure/
remediation: rejected findings return to this producer position for
another pass; this record is not itself promotable until an independent
reviewer signs off.

## Alternatives considered

- **Leave stages 20 and 30 split, just fix their cross-references.**
  Rejected: `BU-P2-003` and `BU-P2-012` both describe one single-message,
  two-call concurrent dispatch; no way to fix the *references* between two
  separately-triggered engine stages recovers that concurrency or removes
  the L4 dependency edge from 30 back to 20 — the defect is the stage
  boundary itself, not a citation.
- **Treat the four missing units (`BU-P2-005/008/011/012`) as out of pilot
  scope, since the current package "already works" without them.**
  Rejected: `BU-P2-013`'s own prompt-assembly instruction requires pasting
  in "the full smell baseline… since the sub-agent has no other access to
  it" — that baseline exists nowhere in the promoted package, so the
  instruction as written cannot be executed faithfully. These are not
  optional enrichments; they were already adjudicated as this package's
  content by the reference corpus's own synthesis and simply never landed.
- **REHOME or SPLIT the whole package.** Rejected: the PL-4 driver
  classification, trigger, and outcome all still correctly describe one
  independent Sergeant workflow with a coherent authority envelope; every
  defect found here is an internal-fidelity gap against the package's
  *own* already-adjudicated source classification (a promotion-execution
  defect), not evidence that `code-review` is the wrong surface for this
  behavior.

## Final disposition
STAND

Package identity, driver, and PL-4 workflow rung are correct and not in
dispute. Internal restructuring is required before promotion (stage merge,
two new workflow-local artifacts, package-level Authority envelope,
per-stage Bounded-judgment sections, one broken pointer) — see the
Behavior-unit dispositions table for the individual HARVEST/FOLD grain
within this STAND. Proposal §8.9 requires draft treatment for any
"generated or substantially rewritten" package regardless of its
top-level modifier, so the corrected content is written under
`docs/gauntlet/runs/icm-r2/code-review/draft/` rather than edited in
place.

## Validation evidence

- Every file under `.sergeant/workflows/code-review/` read in full (§8.3
  Inventory): `CONTEXT.md`, `index.md`, `workflow.toml`, all five stage
  `CONTEXT.md` files, all five stage `output/README.md` files.
- Upstream source read in full and quotes checked directly against
  `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md`.
- Promoted-package citations cross-checked against
  `docs/gauntlet/promoted-provenance/code-review.md`.
- Prior classification records for this package's full behavior-unit set
  (18 units, `BU-P2-001` through `BU-P2-018` minus none) located and read
  in `reference-corpus/behavior-units/P2.ndjson`,
  `reference-corpus/helper-map.md`, `reference-corpus/shared-context-map.md`,
  and `reference-corpus/synthesis.md` — confirming 4 units
  (`BU-P2-005`, `BU-P2-008`, `BU-P2-011`, `BU-P2-012`) were classified with
  a named destination and never materialized, and that
  `reference-corpus/classification-ledger.md` has no entry adjudicating
  them out.
- Confirmed no `.sergeant/workflows/*/workflow.toml` in this repository
  declares more than a flat sequential `stages` list (`grep parallel
  .sergeant/workflows/*/workflow.toml`) — the engine has no concurrent-
  stage primitive, which is why `BU-P2-003`'s concurrency requirement must
  be met *inside* one stage rather than by stage sequencing, supporting
  the merge over any fix that keeps two stages.
- Confirmed the package's `CONTEXT.md` → `provenance.md` reference does
  not resolve to any file under `.sergeant/workflows/code-review/`.
