# 20-implement: implement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-triage-and-route/output/README.md | L4 | upstream artifact produced by `10-triage-and-route` |

## Purpose

The discipline chosen at `10-triage-and-route` runs to its own completion.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

The discipline chosen at `10-triage-and-route` runs to its own completion.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegates to below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Delegates to

Whichever discipline `10-triage-and-route` selected, dispatched as its own separately-admitted Work rather than pulled in as inline text — collapsing a real, checkpointed workflow into this single stage's context would destroy the independent checkpoints and authority envelope each of these packages exists to protect (`docs/icm/convention.md` §4 rule 1; same correction `implement`'s own `code-review` delegation required at ICM-R3):

- **`diagnose-bug`, `prototype`, `implement`, or `deepen-module`** — each a separately-admitted PL-4 workflow with its own settled placement. Dispatch the chosen one as its own Work (`proposal-next-iteration-icm-workflows.md` §7.7); true nested-workflow invocation is the eventual correct fix, filed as an engine-gap claim (`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r3/implement/draft/engine-gap-nested-workflow-invocation.md` — this stage's own delegation pattern is additional source evidence for that same filed claim, not a second claim).
- **`tdd`** — applies `@@tdd` and `@@test-quality` (`.sergeant/common/contexts/`), the same as `implement/10-implement-with-tdd`. `tdd`'s own ICM-R3 REHOME is confirmed by owner ruling after independent review.

## Bounded judgment

Apply `@@bounded-judgment`.

### J3 — settled, not re-litigated here
- Which discipline was selected is a settled decision consumed from `10-triage-and-route`'s own output — this stage does not re-decide it.

### J2 — delegated to this stage
- How to scope the diff/intent handed to the dispatched discipline (or, for the `tdd` branch, applied directly per `@@tdd`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- **Seam confirmation**, when the `tdd` branch applies: `@@tdd` requires no test is written at an unconfirmed seam — stated here explicitly rather than left implicit, per the same hidden-dependency finding `implement/10-implement-with-tdd` closed.
- Substituting this stage's own judgment for a dispatched discipline's own reported outcome.

### Completion boundary
This stage may complete only when the selected discipline reports its own terminal outcome (or, for `tdd`, when every confirmed seam has a red-then-green cycle recorded).

### Decision evidence
Recorded in this stage's own output artifact.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
