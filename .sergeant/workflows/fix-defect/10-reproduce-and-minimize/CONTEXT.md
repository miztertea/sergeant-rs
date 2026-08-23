# 10-reproduce-and-minimize: reproduce and minimize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-build-feedback-loop/output/README.md | L4 | the red-capable command this stage runs |

## Purpose

The loop goes red on the user's exact symptom and every remaining element
is load-bearing. **Hard gate: no edit to the subject happens before this
stage completes.**

## What must become true here (durable outcome)

The loop goes red on the user's exact symptom, and the repro is minimized
until every remaining element is load-bearing — removing any one of them
makes it go green.

## Behavior contract

- **This stage begins by running the loop and watching it go red,
  confirming the bug appears.**
  (trigger: `00`'s completion criterion is met; outcome: the bug is
  confirmed to reproduce via the built loop)
- **The actor must confirm the loop reproduces the specific failure mode
  the user described (not a coincidentally nearby different failure),
  that it reproduces across multiple runs (or at a high enough rate for
  non-deterministic bugs), and that the exact symptom has been captured
  for later verification.**
  (trigger: the loop has gone red; outcome: the reproduction is confirmed
  to be the right bug, not a look-alike)
- **Once red, the repro is shrunk to the smallest scenario that still
  goes red by cutting inputs, callers, config, data, and steps one at a
  time, re-running the loop after each cut, keeping only what is
  load-bearing for the failure.**
  (trigger: the bug reliably reproduces; outcome: a minimal repro is
  produced containing only load-bearing elements)
- **Minimizing the repro shrinks the hypothesis space for `20-hypothesize`
  and produces the clean regression test used later in
  `40-fix-with-regression-test`.**
  (trigger: n/a (rationale); outcome: minimization is understood as
  feeding both hypothesis generation and the eventual regression test)
- **A failed reproduction is a terminal blocked outcome with evidence,
  never an improvised fix.** If the bug cannot be made to reproduce at
  all, this stage does not proceed to a speculative patch; it stops with
  what was tried.
  (trigger: no reproduction can be achieved; outcome: `work.blocked` is
  the honest terminal state, not a guessed fix against an unconfirmed
  bug)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
What to cut, and in what order, while minimizing.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### J5 — governing completion gate
**The actor must not proceed past this stage until the bug has been both
reproduced and minimized** — a governing stage-contract prohibition, not
a discretionary choice. **No edit to the subject under diagnosis happens
before this gate is passed.**

### J0 — must become `needs_input`
Inherits workflow envelope unchanged. This stage's one failure mode — the
bug cannot be reproduced at all despite genuine attempts — is governed by
`J5` above: it is a terminal, evidenced `work.blocked`, never a
`needs_input` escalation asking permission to guess.

### Completion boundary
This stage may complete only when the loop reproduces the user's exact
symptom and every remaining element is load-bearing (removing any one
makes it go green).

### Decision evidence
The minimized repro and what was cut are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
