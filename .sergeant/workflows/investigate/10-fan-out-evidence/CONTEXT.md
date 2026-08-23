# 10-fan-out-evidence: fan out evidence

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-frame/output/frame.md | L4 | the bounded question(s) and stopping condition every seat's sub-question is drawn from |

## Purpose

N isolated evidence seats, spawned in one message, each with its own
bounded sub-question and cap; each returns cited evidence.

**What "isolated" means here, exactly.** These seats are sub-agents
spawned inside one stage execution, not separate Works. Their isolation
is context isolation only: each seat sees the brief this stage hands it
and nothing of its siblings' reasoning. It is weaker than the isolation a
separate Work would give — the seats share this stage's single execution,
its journal entry, its usage window and its failure; there is no per-seat
journal, no per-seat recovery, and a stage that dies takes every seat's
unwritten output with it. `docs/icm/convention.md` §6.3 places review
independence in the execution boundary: this stage has one execution
boundary, not four. Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

## What must become true here (durable outcome)

N isolated evidence seats have each returned cited evidence against their
own bounded sub-question, or been recorded as failed by name.

## Behavior contract

Apply `@@fan-out-evidence`. This package's own narrowing:

- **N is bounded by the sub-questions `00-frame` identified, not by
  ambition.** The framed question(s) determine how many seats are
  spawned, not a target seat count decided independently.
  (trigger: the question is framed; outcome: fan-out width tracks the
  actual question structure)
- **Each seat is asked for primary-source evidence — official docs,
  source code, specs, first-party APIs — rather than secondary summaries,
  with every claim traced back to its owning source.**
  (trigger: assembling a seat's brief; outcome: the resulting evidence is
  independently checkable, not hearsay)
- **Every seat is spawned in the same single message.** A seat spawned
  after reading another's report is a chain, not this mechanism.
  (trigger: dispatching seats; outcome: no seat's findings can bias
  another's search)
- **A seat that cannot complete is recorded as failed by name; this stage
  does not re-run a dead seat itself.**
  (trigger: a seat dies or returns nothing; outcome: `20-synthesize` can
  state honest coverage rather than the stage silently absorbing the
  gap)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How many seats to spawn and how to bound each one's sub-question, within
what `00-frame` established.

### J1 — local choices allowed
Exact invocation wording, so long as every seat is spawned in one
message.

### J0 — must become `needs_input`
Primary sources conflict on a material fact and no higher rung resolves
which one governs; or no primary source can be found for a question the
requester needs answered.

### Completion boundary
This stage may complete only when every spawned seat has either reported
cited evidence or been recorded as failed by name.

### Decision evidence
`output/evidence.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
