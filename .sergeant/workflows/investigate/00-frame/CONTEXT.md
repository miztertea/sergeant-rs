# 00-frame: frame

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The frontier, the bounded question(s), and the stopping condition are
written down.

## What must become true here (durable outcome)

A pinned point of reference (`@@pin-fixed-point`, where the question
concerns code at a specific revision), the bounded question or questions
this investigation answers, and a stated stopping condition — what would
count as "enough evidence" — all exist before any evidence-gathering
begins.

## Behavior contract

- **Apply `@@pin-fixed-point` where the question concerns a specific
  revision of code**: pin it and confirm it resolves before framing
  questions against it. Where the question is not revision-bound (a
  general research question), this does not apply and the stage says so.
  (trigger: this stage begins; outcome: any revision-bound claim later in
  the investigation has one fixed point to be judged against)
- **State the bounded question(s) precisely enough that a reader could
  judge, at the end, whether they were actually answered.** A vague
  question ("investigate the auth system") is sharpened into one or more
  answerable questions before evidence-gathering begins.
  (trigger: the intent names a topic; outcome: `10-fan-out-evidence` has
  concrete sub-questions to bound its seats against, rather than an open
  topic)
- **State the stopping condition**: what would count as enough evidence to
  answer, or to honestly conclude "not yet known." This absorbs
  `wayfinder`'s frontier-mapping concept — naming what's known, what's
  fog, what would resolve it — without recreating its ticket-graph
  mechanics.
  (trigger: the question is stated; outcome: the investigation has a
  named, checkable end condition rather than running until effort runs
  out)
- **This stage frames what the intent already names; it never
  interviews.** Naming a destination/question through live back-and-forth
  is Captain's act (R-NS-6), before dispatch, not this stage's.
  (trigger: the intent under-specifies the question; outcome: this stage
  either sharpens what is already implicit in the intent or escalates —
  it does not open a live interview to construct the question from
  scratch)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Sharpening a vague topic into precise, answerable sub-questions, and
stating a reasonable stopping condition, within what the intent already
implies.

### J1 — local choices allowed
Exact wording of the framing record.

### J0 — must become `needs_input`
The intent does not name a question sharp enough to frame even after
reasonable sharpening, or names none at all — this is a live-interview
gap, not something this stage may fill by inventing a question.

### Completion boundary
This stage may complete only once the bounded question(s) and a stopping
condition are stated, and, where revision-bound, the fixed point is
pinned.

### Decision evidence
`output/frame.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
