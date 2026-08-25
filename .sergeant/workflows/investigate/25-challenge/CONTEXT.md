# 25-challenge: challenge

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-synthesize/output/synthesis.md | L4 | the conclusions this stage attacks |

## Purpose

A refuter pass over the conclusions, defaulting to refuted: a conclusion
stands only if the challenge could not overturn it.

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

Every conclusion in `20-synthesize`'s document has been attacked; a
conclusion is marked standing only where the challenge could not overturn
it, and marked overturned (with the challenge's argument) otherwise.

## Behavior contract

Apply `@@refute`, applied here to conclusions rather than typed findings:

- **Every conclusion arrives presumptively refuted and stays that way
  unless a challenge, having attacked it, reports that it could not
  overturn it and states what it checked.**
  (trigger: this stage begins; outcome: a conclusion must earn standing,
  never inherit it from being written down)
- **The challenge attacks, it does not arbitrate**: it tries to
  reproduce the answer from the cited evidence, finds a conflicting
  primary source, or shows the citation does not actually support the
  claim made from it.
  (trigger: writing the challenge's brief; outcome: the challenge
  produces an argument with evidence, not a vote)
- **Silence, an empty challenge, or an ambiguous verdict leaves the
  conclusion unconfirmed** — it is reported as such in the record, never
  silently promoted to standing.
  (trigger: the challenge fails or hedges; outcome: the stage fails safe)
- **This stage does not add new answers or rewrite `20-synthesize`'s
  document.** A gap the challenge notices becomes a recorded remaining
  unknown for `40-close`, not a silent rewrite of the synthesis.
  (trigger: the challenge surfaces something the synthesis missed;
  outcome: the synthesis stays the synthesis; the gap is named instead)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase the challenge's brief for each conclusion.

### J1 — local choices allowed
Exact invocation wording.

### J0 — must become `needs_input`
The challenge's verdict turns on a decision only a human can make (e.g.
which of two primary sources governs when they conflict on policy, not
fact).

### Completion boundary
Every conclusion in the synthesis has a challenge verdict — standing or
overturned, with the challenge's argument recorded either way.

### Decision evidence
`output/challenge.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
