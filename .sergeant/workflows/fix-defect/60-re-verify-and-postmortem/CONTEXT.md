# 60-re-verify-and-postmortem: re-verify and postmortem

## Inputs

| File | Layer | Why |
|---|---|---|
| ../55-refute/output/findings.md | L4 | the confirmed/refuted finding set from the panel over the fix |
| ../40-fix-with-regression-test/output/fix.md | L4 | the original defect fix this stage re-verifies and, if the panel confirmed further findings, extends |

## Purpose

The fix commits have been re-attacked and their tests audited; the
root-cause record exists: what was actually wrong, how it got in, what
would have caught it.

## What must become true here (durable outcome)

Every `confirmed` finding from `55-refute` is fixed in a named commit or
recorded as unfixed with a reason (`@@fix-confirmed`); the resulting fix
commits — the original defect fix from `40`, plus any commit this stage
adds for a confirmed finding — are re-attacked and their tests audited
(`@@re-verify`); and a root-cause postmortem closes the workflow.

## Behavior contract

Apply `@@fix-confirmed` for any `confirmed` finding from `55-refute`, then
`@@re-verify` over the resulting fix commits. This package's own
narrowing:

- **Only `confirmed` findings from `55-refute` are fixed here; no
  opportunistic change.** A `refuted` finding is not touched.
  (trigger: this stage begins; outcome: any additional fix this stage
  makes is exactly the confirmed panel findings, nothing more)
- **The subject of the re-verify pass is every fix commit that exists by
  this point** — the original fix from `40-fix-with-regression-test`, and
  any commit this stage added for a confirmed finding.
  (trigger: fixes are settled; outcome: the re-attack covers the whole
  fix, not just whatever this stage itself just added)
- **Two passes run: a re-attack for defects the fixes introduced, and a
  test-honesty audit of every test added or changed across those
  commits.**
  (trigger: fix commits are identified; outcome: both measured failure
  classes — a fixer-introduced blocker and a test that proves nothing —
  are looked for by name)
- **A new blocker is a `needs_input` escalation, not a second fix round.**
  There is no loop primitive in this engine.
  (trigger: the re-attack finds a blocker; outcome: the human decides
  whether to extend this Work, rather than the workflow improvising
  another round)
- **Before the postmortem, confirm the closing checklist**: the original
  repro from `10-reproduce-and-minimize` no longer reproduces, the
  regression test passes (or its seam-absence is documented), all tagged
  debug instrumentation from `30-instrument` is removed via a prefix
  grep, and throwaway prototypes are deleted or clearly marked.
  (trigger: re-verify is clean; outcome: the diagnosis is confirmed
  actually closed, not just believed closed)
- **The postmortem — what was actually wrong, how it got in, what would
  have caught it — is written after the fix and re-verify, when the most
  is known, and states the correct hypothesis (from `20`/`30`) for the
  next debugger.**
  (trigger: the closing checklist passes; outcome: a durable root-cause
  record exists, timed deliberately after the fix rather than guessed
  before it)
- **If the answer to "what would have prevented this" involves an
  architectural change (no good test seam, tangled callers, hidden
  coupling), that finding and a recommendation are recorded — never
  dispatched or delegated, only recommended for Captain and the human to
  act on.**
  (trigger: the postmortem implicates architecture; outcome: the
  recommendation is durably recorded without this workflow broadening its
  own scope to include the architectural work)
- **A clean re-verify is recorded as a positive result — what was
  attacked, how, and what was found not to be wrong — never as an empty
  file.**

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- How to fix a confirmed finding, within `@@fix-confirmed`'s bounds.
- How to design the re-attack and test-honesty audit for the specific
  commits in front of it.
- Judging whether the fix implicates an architectural change worth
  flagging.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- A confirmed finding cannot be fixed within the intent's scope.
- A new blocker survives into the fix commits.

### Completion boundary
This stage may complete only when every confirmed finding is fixed or
recorded unfixed with a reason, both re-verify passes have run over every
fix commit, the closing checklist is verified, and the postmortem is
written.

### Decision evidence
`output/re-verify-and-postmortem.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
