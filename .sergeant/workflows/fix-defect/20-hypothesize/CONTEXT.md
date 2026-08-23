# 20-hypothesize: hypothesize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-reproduce-and-minimize/output/README.md | L4 | the minimized repro this stage generates hypotheses against |

## Purpose

A causal hypothesis is written down before instrumenting, with what would
disprove it.

## What must become true here (durable outcome)

3-5 ranked falsifiable hypotheses are recorded before any instrumentation
begins.

## Behavior contract

- **Before testing any of them, the actor generates 3-5 ranked
  hypotheses, because generating only one hypothesis anchors reasoning on
  the first plausible idea.**
  (trigger: the bug is reproduced and minimized; outcome: a ranked,
  multi-hypothesis list exists before any hypothesis is tested)
- **Each hypothesis must be falsifiable, stated in the form "If `<X>` is
  the cause, then changing `<Y>` will make the bug disappear / changing
  `<Z>` will make it worse"; if no prediction can be stated, the
  hypothesis is a vibe to be discarded or sharpened.**
  (trigger: generating hypotheses; outcome: only falsifiable,
  prediction-bearing hypotheses survive into `30-instrument`)
- **The ranked hypothesis list should be shown to the user before testing
  begins, since users often re-rank instantly from domain knowledge or
  already-ruled-out hypotheses; this is a cheap checkpoint that should
  not block progress if the user is away.**
  (trigger: hypotheses are ranked and falsifiable; outcome: the user gets
  a chance to re-rank or rule out hypotheses without blocking the actor
  if unavailable)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Generating and ranking 3-5 falsifiable hypotheses.
- Proceeding without the user's re-ranking if the user is unavailable —
  an explicitly named non-blocking exception, not a J0 escalation.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- A hypothesis that cannot be stated as a falsifiable prediction —
  discard or sharpen it before showing the list, rather than presenting a
  vibe as a hypothesis.
- Two hypotheses are equally supported and distinguishing them needs a
  decision only the human can make (e.g. which of two plausible root
  causes to prioritize investigating first, when instrumentation cost is
  material).

### Completion boundary
This stage may complete only when 3-5 ranked, falsifiable hypotheses have
been shown to the user (or, if unavailable, recorded for later
re-ranking).

### Decision evidence
The ranked hypothesis list is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
