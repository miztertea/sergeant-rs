# Code Review
Draft workflow package — candidate **W24** `code-review` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
Revised at ICM-R2
(`docs/gauntlet/runs/icm-r2/code-review/adjudication-draft.md`) per
`reference/proposal-icm-r-procedure-authority.md` §8. This is Layer 1
orientation only — it is never delivered as a stage's instructions; each
stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`docs/icm/convention.md` §1a rule 5).

## Purpose

Review a diff on two parallel, non-contaminating axes, reported side by
side:

- **Standards** — does the code conform to this repo's documented coding
  standards?
- **Spec** — does the code faithfully implement the originating issue,
  PRD, or spec?

## Trigger

A diff needs review before merge (invoked directly or delegated from
`worker-mission`/`implement`).

## Authority envelope

This workflow receives an already-admitted Work intent (a diff to review).

### Workflow may decide
- Which fixed comparison point to confirm as valid, given the user's
  input (`00-pin-fixed-point`).
- Where to look for and how to select the spec source, within the fixed
  priority order (`10-identify-spec-source`).
- How to phrase the Standards/Spec sub-agent prompts within the bounds of
  the recorded briefs, and which baseline smells rise to reportable given
  the diff (`20-30-parallel-review`).
- How to lightly clean, without altering the substance of, the two
  sub-agent reports before presenting them (`40-aggregate`).

### Workflow may not decide
- Whether to skip asking the user for the fixed point when none is given
  — J0, `00-pin-fixed-point`.
- Whether to merge or rerank the Standards and Spec axes — J5 governing
  constraint, never merge.
- Whether to suppress a documented repo standard in favor of the smell
  baseline — J5, the repo's own documented standard always overrides.

### Human or Captain gates
- Naming the fixed comparison point when the user did not supply one.
- Naming the spec source when none is discoverable by the fixed priority
  order.

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact (Layer 4, `evidence` disposition) per `.sergeant/common/
contexts/bounded-judgment.md` §Decision evidence.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-pin-fixed-point` | actor-stage (§6.4, judgment) | The fixed point resolves and the diff is non-empty, or this fails here rather than inside a sub-review. |
| `10-identify-spec-source` | actor-stage (§6.4, judgment) | The spec source is identified via a fixed priority order ending in asking the user. |
| `20-30-parallel-review` | actor-stage (§6.4, judgment) | Both axes run as isolated sub-agents, spawned in a single message, and each produces its own report. |
| `40-aggregate` | actor-stage (§6.4, judgment) | The two axes are reported separately, never merged or reranked. |

## Notes for reviewers

The two-axis separation is the durable design point, not the
sub-agent mechanism that happens to isolate the two reviews from each
other. `20-30-parallel-review` merges what were two sequential stages in
the prior revision — the reference corpus's own classification record
already named this stage `20-30-parallel-review`; splitting it
into two engine stages could not represent the source's required
single-message, two-call concurrent dispatch.

## Provenance

See `docs/gauntlet/promoted-provenance/code-review.md` for the prior
revision's complete stage-to-behavior-unit mapping and workflow-level
citations, and `docs/gauntlet/runs/icm-r2/code-review/adjudication-draft.md`
for this revision's full behavior-unit disposition table.
