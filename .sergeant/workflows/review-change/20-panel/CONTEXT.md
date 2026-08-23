# 20-panel: panel

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-pin-fixed-point/output/README.md | L4 | the pinned revision and diff command every seat's brief is built from |
| ../10-identify-spec-source/output/README.md | L4 | the spec source (or its recorded absence) every seat's brief includes |
| references/smell-baseline.md | L3 | the fixed Fowler-smell baseline pasted into the simplicity seat's brief |

## Purpose

Four axis seats have reported; every finding is recorded in the typed set
at `status: raised`.

**What "isolated" means here, exactly.** These seats are sub-agents
spawned inside one stage execution, not separate Works. Their isolation
is context isolation only: each seat sees the brief this stage hands it
and nothing of its siblings' reasoning. It is weaker than the isolation a
separate Work would give — the seats share this stage's single execution,
its journal entry, its usage window and its failure; there is no per-seat
journal, no per-seat recovery, and a stage that dies takes every seat's
unwritten output with it. `docs/icm/convention.md` §6.3 places review
independence in the execution boundary: this stage has one execution boundary, not four.
Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

## What must become true here (durable outcome)

Four axis seats — spec-fidelity, invariants, simplicity, test-honesty —
have each reported (or been recorded as failed by name), and every
finding any of them raised is in the typed set at `status: raised`.

## Behavior contract

Apply `@@panel` and `@@fan-out-evidence`. This package's own narrowing:

- **The four axis seats are spawned as sub-agents in a single message, in
  parallel, so no seat's context pollutes another's; this stage judges
  nothing on its own and fixes nothing (this workflow has no fixer
  stage).**
  (trigger: the fixed point and spec source are available; outcome: four
  independent briefs are produced concurrently, and this stage is a
  collector)
- **The simplicity seat's brief includes the full smell baseline
  (`references/smell-baseline.md`) pasted in, and the rule that a
  documented repo standard always overrides a baseline smell.**
  (trigger: assembling the simplicity seat's brief; outcome: the seat
  applies the same repo-overrides-baseline rule the retired `code-review`
  package's Standards axis used)
- **The four axes are never merged, never re-ranked, and never traded off
  against one another.**
  (trigger: a finding could plausibly sit on two axes; outcome: it is
  recorded once, under the raising seat's axis)
- **Every finding this stage records enters the typed set at
  `status: raised`. This stage never writes `confirmed`.**
  (trigger: transcribing a seat's report; outcome: confirmation is
  `25-refute`'s act alone)
- **A seat that cannot complete degrades the panel to fewer axes, named in
  `output/findings.md` and carried into the final report.**
  (trigger: a seat returns nothing, errors, or is not spawned; outcome:
  reduced coverage is reported as reduced)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase each seat's brief within the bounds above.

### J1 — local choices allowed
Exact invocation wording, so long as all four seats are spawned in one
message.

### J0 — must become `needs_input`
None beyond what `10-identify-spec-source` already resolved — a missing
spec source was recorded there, not re-litigated here; the spec-fidelity
seat is briefed with that recorded absence rather than this stage
inventing a standard.

### Completion boundary
This stage may complete only when every spawned seat has either reported
or been recorded as failed by name.

### Decision evidence
`output/findings.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
