# 60-cleanup-and-postmortem: cleanup and postmortem

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-fix-with-regression-test/output/README.md | L4 | upstream artifact produced by `50-fix-with-regression-test` |

## Purpose

Repro gone, test passing, instrumentation removed, hypothesis recorded, architectural hand-off if warranted.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

Repro gone, test passing, instrumentation removed, hypothesis recorded, architectural hand-off if warranted.

## Behavior contract

- **Before declaring the diagnosis done, the actor must confirm: the original repro no longer reproduces, the regression test passes (or the seam absence is documented), all `[DEBUG-...]` instrumentation is removed via a prefix grep, throwaway prototypes are deleted or clearly marked, and the correct hypothesis is stated in the commit/PR message for the next debugger.**
  (trigger: a fix has been applied; outcome: a fixed set of closing conditions is verified before the diagnosis is considered complete)
  — `BU-P2-048`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 6: required checklist, lines 128-132)
- **After the fix is in, the actor asks what would have prevented the bug; if the answer involves architectural change (no good test seam, tangled callers, hidden coupling) the actor hands off to the `/improve-codebase-architecture` skill with specifics, making the recommendation only after the fix — not before, since more is known by then.**
  (trigger: the fix and cleanup checklist are complete; outcome: an architectural-improvement recommendation is optionally handed off to a separate workflow, timed deliberately after the fix)
  — `BU-P2-049`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 6: architecture handoff, lines 134-134)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
