# Implement
Draft workflow package — candidate **W23** `implement` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
Revised at ICM-R3
(`docs/gauntlet/runs/icm-r3/implement/adjudication-draft.md`) per
`reference/proposal-icm-r-procedure-authority.md` §8. This is Layer 1
orientation only — it is never delivered as a stage's instructions; each
stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`docs/icm/convention.md` §1a rule 5).

## Purpose

Implement a piece of work from a spec or ticket set, explicit-invocation-only.

## Trigger

Explicitly invoked to implement a defined piece of work (never auto-loaded).

## Authority envelope

This workflow receives an already-admitted Work intent (a spec or ticket
set to implement).

### Workflow may decide
- Which concrete seam, test-framework idiom, and minimal implementation
  shape to choose within one confirmed TDD cycle (`10-implement-with-tdd`).
- How to phrase and scope the review dispatch to `code-review`, and how to
  act on its findings, within `code-review`'s own reported disposition
  (`30-review`).
- Ordering of otherwise-equivalent confirmed seams; local test file layout
  and commit message content (local, reversible choices, both stages).

### Workflow may not decide
- Whether to auto-load itself because a task merely looks like
  implementation — J5: explicit-invocation-only, never triggered
  implicitly.
- Whether to write a test at an unconfirmed seam — J0, inherited from the
  `@@tdd` discipline this workflow applies in `10-implement-with-tdd` (see
  that stage's own Bounded judgment section).
- Whether to merge, rerank, or skip either axis of `code-review`'s
  Standards/Spec report, or to substitute its own judgment for a review it
  did not itself perform — J5, inherited from `code-review`'s own settled
  Authority envelope (`.sergeant/workflows/code-review/CONTEXT.md`).

### Human or Captain gates
- Confirming seams before the first test of a TDD cycle, when the loading
  stage cannot responsibly infer them from the ticket alone.
- Any finding `code-review` itself routes to the user rather than resolving
  (its own `10-identify-spec-source` "no spec found" case, its own
  `00-pin-fixed-point` "no fixed point given" case) — these are gates
  `code-review` owns as a separately-admitted execution, not gates this
  workflow re-implements.

### Decision record
Material decisions cite J-rungs inline in each stage's own output artifact
(Layer 4) per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-implement-with-tdd` | actor-stage (§6.4, judgment) | Implementation proceeds seam by seam; folds the demoted `20-verify` checkpoint as a helper (N1 adjudication A4). |
| `30-review` | actor-stage (§6.4, judgment) | The change is reviewed; folds the demoted `40-commit` checkpoint as a helper (N1 adjudication A4). |

`20-verify` and `40-commit` were demoted per N1 adjudication A4 (finding N1-BH-02): both were classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate. Their behavior units survive, folded into the adjacent judgment-bearing stage as helper invocations — see each stage's own `CONTEXT.md` and `docs/gauntlet/promoted-provenance/implement.md`'s "Adjudication A4" section.

## Relationships to other workflows

- `10-implement-with-tdd` applies `@@tdd` and `@@test-quality`
  (`.sergeant/common/contexts/`) — `tdd`'s own ICM-R3 REHOME, confirmed by
  owner ruling after independent review.
- `30-review` delegates to **code-review**, dispatched as its own
  separately-admitted Work (not context composition — see that stage's own
  `CONTEXT.md`). `code-review`'s own placement is settled (STAND, ICM-R2).

## Notes for reviewers

Explicit-invocation-only — this workflow must never be auto-loaded merely because the task looks like implementation; this rule is mirrored cross-harness as well.

ICM-R3 revision (`docs/gauntlet/runs/icm-r3/implement/adjudication-draft.md`):
both stages gain this `## Authority envelope` companion and their own
`## Bounded judgment` sections, replacing the prior generic
`## Judgment required` boilerplate (`docs/icm/convention.md` §6.1,
`docs/adr/0013` decision 4). The `30-review` stage's delegation to
`code-review` is corrected from "context composition today" to "dispatched
as a separate Work" — `code-review` is a real four-stage, authority-
enveloped workflow and cannot be faithfully represented by pulling its text
into this workflow's own stage context. A full engine-gap claim for real
nested/child workflow invocation is filed at
`docs/gauntlet/runs/icm-r3/implement/draft/
engine-gap-nested-workflow-invocation.md`, evidenced by this package's own
two delegations (both currently hedge "context composition today... does
not exist yet" without the claim record §5 requires).

## Provenance

See `docs/gauntlet/promoted-provenance/implement.md` for the complete
stage-to-behavior-unit mapping and workflow-level citations. (ICM-R3
correction: the prior text pointed at a workflow-local `provenance.md`
that does not exist under `.sergeant/workflows/implement/` — the archived
file lives under `docs/gauntlet/promoted-provenance/`, matching every
other promoted package's own layout.)
