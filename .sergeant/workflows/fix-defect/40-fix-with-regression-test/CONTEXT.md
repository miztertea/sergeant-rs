# 40-fix-with-regression-test: fix with regression test

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-instrument/output/README.md | L4 | the confirmed cause this fix is built against |

## Purpose

The regression test exists and fails against the unfixed code first;
then the fix; then it passes.

## What must become true here (durable outcome)

A test exists at a correct seam before the fix, watched failing then
passing — or the seam's absence is recorded as the finding.

## Behavior contract

Apply `@@test-first` for the seam-confirmation discipline. This package's
own narrowing:

- **The regression test is written before the fix, but only if there is
  a correct seam for it.**
  (trigger: a fix is about to be applied; outcome: test-first discipline
  is applied conditionally on seam availability)
- **A correct seam is one where the test exercises the real bug pattern
  as it occurs at the call site; a too-shallow seam gives false
  confidence.**
  (trigger: deciding whether a seam is adequate for a regression test;
  outcome: the actor distinguishes a load-bearing test seam from a
  misleading one)
- **If no correct seam exists, that absence is itself the finding: it is
  noted as evidence that the codebase architecture is preventing the bug
  from being locked down, and flagged for `60-re-verify-and-postmortem`.**
  (trigger: no correct test seam is available; outcome: the seam gap is
  recorded as a durable finding rather than silently skipped)
- **When a correct seam exists: turn the minimized repro into a failing
  test at that seam, watch it fail, apply the fix, watch it pass, then
  re-run the `00-build-feedback-loop` loop against the original
  un-minimized scenario.**
  (trigger: a correct seam has been identified; outcome: the fix is
  proven both at the minimal seam and against the original full scenario)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Judging whether a candidate seam is load-bearing or too shallow.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### Required fallback (not J0 — a named outcome, not an escalation)
No correct seam exists. Record the absence as the finding rather than
silently skipping — this is a required disposition the contract itself
names, not a decision to ask the user about.

### J0 — must become `needs_input`
The fix requires a scope or policy change beyond what the intent
authorized.

### Completion boundary
This stage may complete only when either a regression test exists at a
correct seam, watched failing then passing, with the original feedback
loop re-run against the original scenario — or the seam's absence is
recorded as the finding.

### Decision evidence
The regression test (or the recorded seam-absence finding) is this
stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
