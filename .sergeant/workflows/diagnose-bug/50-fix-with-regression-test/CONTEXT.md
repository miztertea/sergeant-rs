# 50-fix-with-regression-test: fix with regression test

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-instrument/output/README.md | L4 | upstream artifact produced by `40-instrument` |

## Purpose

A test exists at a correct seam before the fix, or the seam's absence is recorded as the finding.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

A test exists at a correct seam before the fix, or the seam's absence is recorded as the finding.

## Behavior contract

- **The regression test is written before the fix, but only if there is a correct seam for it.**
  (trigger: a fix is about to be applied; outcome: test-first discipline is applied conditionally on seam availability)
  — `BU-P2-044`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5, lines 110-110)
- **A correct seam is one where the test exercises the real bug pattern as it occurs at the call site; a too-shallow seam (single-caller test for a multi-caller bug, a unit test that can't replicate the triggering chain) gives false confidence.**
  (trigger: deciding whether a seam is adequate for a regression test; outcome: the actor distinguishes a load-bearing test seam from a misleading one)
  — `BU-P2-045`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5: correct seam definition, lines 112-112)
- **If no correct seam exists, that absence is itself the finding: it must be noted as evidence that the codebase architecture is preventing the bug from being locked down, and flagged for the next phase.**
  (trigger: no correct test seam is available; outcome: the seam gap is recorded as a durable finding rather than silently skipped)
  — `BU-P2-046`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5: no seam finding, lines 114-114)
- **When a correct seam exists, the procedure is: turn the minimized repro into a failing test at that seam, watch it fail, apply the fix, watch it pass, then re-run the Phase 1 feedback loop against the original un-minimized scenario.**
  (trigger: a correct seam has been identified; outcome: the fix is proven both at the minimal seam and against the original full scenario)
  — `BU-P2-047`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5: procedure when a seam exists, lines 116-122)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether a candidate seam is load-bearing or too shallow (`BU-P2-045`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### Required fallback (not J0 — a named outcome, not an escalation)
- **No correct seam exists.** Record the absence as the finding rather than silently skipping (`BU-P2-046`) — this is a required disposition the contract itself names, not a decision to ask the user about.

### Completion boundary
This stage may complete only when either a regression test exists at a correct seam, watched failing then passing, with the Phase 1 loop re-run against the original scenario (`BU-P2-047`) — or the seam's absence is recorded as the finding.

### Decision evidence
The regression test (or the recorded seam-absence finding) is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
