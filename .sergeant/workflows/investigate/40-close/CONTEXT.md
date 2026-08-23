# 40-close: close

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-frame/output/frame.md | L4 | the original question(s) and stopping condition this packet is judged against |
| ../25-challenge/output/challenge.md | L4 | which conclusions stand and which were overturned |
| ../30-record/output/README.md | L4 | the durable artifact's path, cited in the close packet |

## Purpose

Answer, confidence, contradictory evidence found, remaining unknowns, and
recommended next intents.

## What must become true here (durable outcome)

A close packet exists per `@@close` and `@@evidence-requirements`: the
answer (or an honest "not yet known"), a confidence statement, any
contradictory evidence found along the way, remaining unknowns, and
recommended next intents.

## Behavior contract

Apply `@@close` and `@@evidence-requirements`. This package's own
narrowing:

- **State the answer against the exact question(s) framed at `00-frame`**
  — not a broader or narrower question than was actually asked.
  (trigger: the investigation is complete; outcome: the close packet is
  checkable against the original framing, not a retrospective
  restatement of it)
- **"No answer, here is what is known and what would settle it" is a
  valid, complete outcome.** A stopping condition met without a full
  answer is not a failure to paper over.
  (trigger: the stopping condition from `00-frame` is reached without a
  full answer; outcome: the packet says so honestly rather than
  overstating confidence)
- **State confidence, and name any contradictory evidence found along the
  way** — a conflict between seats, or a conclusion `25-challenge`
  overturned.
  (trigger: assembling the packet; outcome: a reader can judge how much
  weight the answer deserves)
- **Recommended next intents are recommendations Captain may act on,
  never work this Work has already decided to do.**
  (trigger: the synthesis's recommendations section suggests further
  work; outcome: scope stays exactly what `00-frame` bounded it to)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase the answer, confidence, and remaining-unknowns sections.

### J1 — local choices allowed
Formatting and ordering of the packet.

### J0 — must become `needs_input`
None beyond what earlier stages already resolved — a J0 encountered
upstream is carried forward and restated here, not re-litigated.

### Completion boundary
This stage may complete only once the packet states an answer or an
honest "not yet known," confidence, contradictory evidence, remaining
unknowns, and recommended next intents.

### Decision evidence
`output/close-packet.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
