# 20-panel: panel

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-orient/output/orientation.md | L4 | the pinned revision, diff command, and spec/acceptance source every seat's brief is built from |
| ../10-implement/output/implementation.md | L4 | the commits under review |
| ../15-validate/output/validation.md | L4 | the validation record every seat's brief includes |

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
unwritten output with it. `.sergeant/common/contexts/icm-policy.md` §6.3 places review
independence in the execution boundary: this stage has one execution boundary, not four.
Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

## What must become true here (durable outcome)

Four axis seats — spec-fidelity, invariants, simplicity, test-honesty —
have each reported (or been recorded as failed by name), and every
finding any of them raised is in the typed set at `status: raised`.

## Behavior contract

Apply `@@panel` and `@@fan-out-evidence`. This package's own narrowing:

- **The four axis seats — spec-fidelity, invariants, simplicity,
  test-honesty — are spawned as sub-agents in a single message, in
  parallel, so no seat's context pollutes another's; this stage judges
  nothing on its own and fixes nothing.**
  (trigger: the implementation and its validation record are available;
  outcome: four independent briefs are produced concurrently, and this
  stage is a collector, not a fifth reviewer)
- **Each seat's brief is self-contained and carries: the pinned revision
  and the exact diff command, the spec or acceptance source located at
  `00-orient` (path or pasted contents), the seat's own axis definition
  verbatim, the finding-record columns, and a 400-word cap. A seat has no
  other access to this Work.**
  (trigger: spawning a seat; outcome: the seat can produce a bounded,
  well-formed report without reaching for context it does not have)
- **The four axes are never merged, never re-ranked, and never traded off
  against one another. A finding belongs to exactly one axis — the seat
  that raised it.**
  (trigger: a finding could plausibly sit on two axes; outcome: it is
  recorded once, under the raising seat's axis, and the other axis is not
  invited to re-litigate it)
- **Every finding this stage records enters the typed set at
  `status: raised`. This stage never writes `confirmed`.**
  (trigger: transcribing a seat's report into `output/findings.md`;
  outcome: confirmation is `25-refute`'s act alone, and the default-refuted
  posture is preserved end to end)
- **A seat that cannot complete degrades the panel to fewer axes, and the
  missing axis is named in `output/findings.md` and carried into the
  close packet. This stage may not re-run a dead seat's axis itself.**
  (trigger: a seat returns nothing, errors, or is not spawned; outcome:
  reduced coverage is reported as reduced, never silently absorbed)
- **The verbatim seat reports are written to `output/findings.md` beneath
  the typed table, tagged by axis, unedited except for formatting.**
  (trigger: the seats have reported; outcome: a later reader can check the
  table against what the seats actually said)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase each seat's brief within the bounds above, and which of a
seat's remarks are findings rather than commentary.

### J1 — local choices allowed
Exact invocation wording, so long as all four seats are spawned in one
message.

### J0 — must become `needs_input`
No spec or acceptance source was located at `00-orient` *and* the intent
does not state acceptance: stop and ask rather than inventing the
standard the change is judged against.

### Completion boundary
This stage may complete only when every spawned seat has either reported
or been recorded as failed by name, and every reported finding is in the
table at `status: raised`.

### Decision evidence
`output/findings.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
