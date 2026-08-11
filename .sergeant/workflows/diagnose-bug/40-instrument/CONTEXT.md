# 40-instrument: instrument

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-hypothesize/output/README.md | L4 | upstream artifact produced by `30-hypothesize` |

## Purpose

One probe per prediction, one variable at a time, tagged logs.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

One probe per prediction, one variable at a time, tagged logs.

## Behavior contract

- **Each instrumentation probe in Phase 4 must map to a specific prediction from Phase 3, changing exactly one variable at a time.**
  (trigger: hypotheses exist from Phase 3; outcome: instrumentation is targeted and interpretable rather than diffuse)
  — `BU-P2-040`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4, lines 96-96)
- **Instrumentation tool preference is ordered: debugger/REPL inspection where the environment supports it (one breakpoint beats ten logs), then targeted logs at boundaries that distinguish hypotheses, and never 'log everything and grep'.**
  (trigger: choosing an instrumentation method; outcome: the least diffuse tool available is chosen first)
  — `BU-P2-041`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4: tool preference, lines 100-102)
- **Every debug log must be tagged with a unique prefix (e.g. `[DEBUG-a4f2]`) so cleanup at the end becomes a single grep; untagged logs survive cleanup, tagged ones die.**
  (trigger: adding temporary debug logging; outcome: all temporary logging is grep-removable by construction)
  — `BU-P2-042`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4: log tagging, lines 104-104)
- **For performance regressions, logs are usually the wrong tool; instead the actor establishes a baseline measurement (timing harness, `performance.now()`, profiler, query plan) and then bisects — measure first, fix second.**
  (trigger: diagnosing a performance regression specifically; outcome: a measured baseline exists before any fix is attempted)
  — `BU-P2-043`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4: performance branch, lines 106-106)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
