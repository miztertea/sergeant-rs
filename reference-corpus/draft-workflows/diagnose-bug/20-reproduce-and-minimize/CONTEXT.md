# 20-reproduce-and-minimize: reproduce and minimize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-build-feedback-loop/output/README.md | L4 | upstream artifact produced by `10-build-feedback-loop` |

## Purpose

The loop goes red on the user's exact symptom and every remaining element is load-bearing.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

The loop goes red on the user's exact symptom and every remaining element is load-bearing.

## Behavior contract

- **Phase 2 begins by running the loop and watching it go red, confirming the bug appears.**
  (trigger: Phase 1's completion criterion is met; outcome: the bug is confirmed to reproduce via the built loop)
  — `BU-P2-031`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2 start, lines 64-64)
- **The actor must confirm the loop reproduces the specific failure mode the user described (not a coincidentally nearby different failure), that it reproduces across multiple runs (or at a high enough rate for non-deterministic bugs), and that the exact symptom has been captured for later verification.**
  (trigger: the loop has gone red; outcome: the reproduction is confirmed to be the right bug, not a look-alike)
  — `BU-P2-032`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2 confirm checklist, lines 68-70)
- **Once red, the repro is shrunk to the smallest scenario that still goes red by cutting inputs, callers, config, data, and steps one at a time, re-running the loop after each cut, keeping only what is load-bearing for the failure.**
  (trigger: the bug reliably reproduces; outcome: a minimal repro is produced containing only load-bearing elements)
  — `BU-P2-033`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2: Minimise, lines 74-74)
- **Minimizing the repro shrinks the hypothesis space for Phase 3 and produces the clean regression test used later in Phase 5.**
  (trigger: n/a (rationale); outcome: minimization is understood as feeding both hypothesis generation and the eventual regression test)
  — `BU-P2-034`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2: Minimise rationale, lines 76-76)
- **Minimization is done when every remaining element is load-bearing — removing any one of them makes the loop go green.**
  (trigger: minimizing the repro; outcome: a precise, testable stopping condition for minimization is applied)
  — `BU-P2-035`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2: Minimise completion, lines 78-78)
- **The actor must not proceed past Phase 2 until the bug has been both reproduced and minimized.**
  (trigger: reproduction is confirmed but minimization is incomplete, or vice versa; outcome: the actor is blocked from Phase 3 until both conditions hold)
  — `BU-P2-036`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2 closing rule, lines 80-80)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
