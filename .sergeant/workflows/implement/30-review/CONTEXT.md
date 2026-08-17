# 30-review: review

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-implement-with-tdd/output/README.md | L4 | upstream artifact produced by `10-implement-with-tdd` (folds the demoted `20-verify` checkpoint, N1 adjudication A4) |

## Purpose

The change is reviewed.

Trigger (workflow-level): Explicitly invoked to implement a defined piece of work (never auto-loaded).

## What must become true here (durable outcome)

The change is reviewed.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegates to below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Delegates to

`code-review`: once implementation is done, the code-review skill/workflow
is used to review the work. `code-review` is a separately-admitted PL-4 workflow —
its own ICM-R2 adjudication (`docs/gauntlet/runs/icm-r2/code-review/
adjudication-draft.md`) settled `STAND`, and it is live today as four real
actor stages (`00-pin-fixed-point`, `10-identify-spec-source`,
`20-30-parallel-review`, `40-aggregate`) with its own
`## Authority envelope` (`.sergeant/workflows/code-review/CONTEXT.md`).

**ICM-R3 correction:** the prior text here described this delegation as
"context composition today... which does not exist yet" — the same hedge
used for a shared-technique delegation elsewhere in this workflow. That
description does not hold for `code-review` specifically: pulling its
four-stage, review-independence-bearing procedure into this single stage's
own context would collapse four fresh executions into one and destroy the
property `code-review`'s own stage split exists to protect
(`docs/icm/convention.md` §4 rule 1 treats exactly this pattern — using a
context reference to imply "and then run that other procedure as a
sub-workflow" — as a scope violation). Until a true nested/child-workflow
capability exists (filed as an engine-gap claim at
`docs/gauntlet/runs/icm-r3/implement/draft/
engine-gap-nested-workflow-invocation.md`), the current best-available
mechanism is to dispatch `code-review` as its own separately-admitted Work
against the diff produced by `10-implement-with-tdd`
(`proposal-next-iteration-icm-workflows.md` §7.7: an agent "could even
submit another `sgt run`" — this loses parent/child trajectory identity,
recovery, and unified telemetry, but at minimum preserves `code-review`'s
own four checkpoints and authority envelope, which context composition
would not).

## Helper: commit (folded from demoted `40-commit`, N1 adjudication A4)

`40-commit` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as the concluding helper invocation of this checkpoint, subordinate to this stage's own judgment-bearing outcome:

- **The final step of implement is to commit the work to the current branch.**
  (trigger: the work has been implemented, verified, and reviewed; outcome: the change is committed to the current branch)

## Bounded judgment

Apply `@@bounded-judgment`. This section carries the local specialization
this stage contributes; it inherits the workflow's own `## Authority
envelope` (`../CONTEXT.md`) unchanged except where narrowed below.

### J2 — delegated to this stage
- How to scope the diff and intent handed to the dispatched `code-review`
  Work.
- How to act on `code-review`'s reported disposition (accept and proceed
  to commit; address findings and re-dispatch) — within the bounds of what
  `code-review` itself reported, not by re-deciding a finding `code-review`
  already resolved.
- Commit message content, once the change is accepted.

### J1 — local choices allowed
- Local sequencing of addressing multiple non-blocking `code-review`
  findings before re-dispatch.

### J0 — must become `needs_input`
- **Substituting this stage's own judgment for a review it did not
  perform.** This stage may not merge, rerank, skip, or overrule either
  axis of `code-review`'s Standards/Spec report — that is `code-review`'s
  own J5 governing constraint (`.sergeant/workflows/code-review/
  CONTEXT.md` "Workflow may not decide"), inherited unchanged because the
  corrected dispatch mechanism above keeps `code-review` a separately-
  admitted execution rather than collapsing it into this stage's own
  context.
- A `code-review` finding that itself surfaces a scope, acceptance, or
  authority question this stage cannot resolve from the recorded Work
  intent — ask, do not guess.

### Completion boundary
This stage may complete only when `code-review` has reported a disposition
for the current diff, any required remediation has been applied and
re-reviewed, and the accepted change has been committed to the current
branch.

### Decision evidence
Write material decisions (how `code-review` findings were acted on, the
commit made) to this stage's own output artifact per
`@@bounded-judgment`'s recommended table shape.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
