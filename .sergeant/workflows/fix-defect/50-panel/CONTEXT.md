# 50-panel: panel

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-build-feedback-loop/output/README.md | L4 | the pinned reproduction command every seat's brief can reference |
| ../40-fix-with-regression-test/output/fix.md | L4 | the fix commits under review |

## Purpose

Four axis seats have reported on the fix; every finding is recorded at
`status: raised`.

**What "isolated" means here, exactly.** These seats are sub-agents
spawned inside one stage execution, not separate Works. Their isolation
is context isolation only: each seat sees the brief this stage hands it
and nothing of its siblings' reasoning. It is weaker than the isolation a
separate Work would give — the seats share this stage's single execution,
its journal entry, its usage window and its failure; there is no per-seat
journal, no per-seat recovery, and a stage that dies takes every seat's
unwritten output with it. `.sergeant/common/contexts/icm-policy.md` §6.3 places review
independence in the execution boundary: this stage has one execution boundary, not four.
Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

## What must become true here (durable outcome)

Four axis seats — spec-fidelity, invariants, simplicity, test-honesty —
have each reported (or been recorded as failed by name) on the fix, and
every finding any of them raised is in the typed set at `status: raised`.

## Behavior contract

Apply `@@panel` and `@@fan-out-evidence`. This package's own narrowing:

- **The four axis seats review the fix commits from
  `40-fix-with-regression-test`, spawned as sub-agents in a single
  message, in parallel.**
  (trigger: the fix and its regression test exist; outcome: four
  independent briefs are produced concurrently)
- **The spec-fidelity axis is judged against the original defect report
  and, where instrumentation confirmed one, the proven root cause from
  `30-instrument` — not a spec document, since a fix's "spec" is the bug
  it was meant to close.**
  (trigger: assembling the spec-fidelity seat's brief; outcome: the seat
  has a concrete standard — did the fix address the confirmed cause? —
  rather than an invented one)
- **Every finding enters at `status: raised`; this stage never writes
  `confirmed`.**
  (trigger: transcribing a seat's report; outcome: confirmation is
  `55-refute`'s act alone)
- **A seat that cannot complete degrades the panel to fewer axes, named
  in `output/findings.md`.**
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
None beyond what earlier stages already resolved — a seam-absence finding
from `40` is carried into the spec-fidelity seat's brief as recorded
evidence, not re-litigated here.

### Completion boundary
This stage may complete only when every spawned seat has either reported
or been recorded as failed by name.

### Decision evidence
`output/findings.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
