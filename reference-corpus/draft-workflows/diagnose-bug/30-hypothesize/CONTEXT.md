# 30-hypothesize: hypothesize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-reproduce-and-minimize/output/README.md | L4 | upstream artifact produced by `20-reproduce-and-minimize` |

## Purpose

3-5 ranked falsifiable hypotheses are shown to the user.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

3-5 ranked falsifiable hypotheses are shown to the user.

## Behavior contract

- **Before testing any of them, the actor generates 3-5 ranked hypotheses, because generating only one hypothesis anchors reasoning on the first plausible idea.**
  (trigger: the bug is reproduced and minimized; outcome: a ranked, multi-hypothesis list exists before any hypothesis is tested)
  — `BU-P2-037`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 3, lines 84-84)
- **Each hypothesis must be falsifiable, stated in the form 'If <X> is the cause, then <changing Y> will make the bug disappear / <changing Z> will make it worse'; if no prediction can be stated, the hypothesis is a vibe to be discarded or sharpened.**
  (trigger: generating hypotheses; outcome: only falsifiable, prediction-bearing hypotheses survive into Phase 4)
  — `BU-P2-038`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 3: falsifiability, lines 86-88)
- **The ranked hypothesis list should be shown to the user before testing begins, since users often re-rank instantly from domain knowledge or already-ruled-out hypotheses; this is a cheap checkpoint that should not block progress if the user is away.**
  (trigger: hypotheses are ranked and falsifiable; outcome: the user gets a chance to re-rank or rule out hypotheses without blocking the actor if unavailable)
  — `BU-P2-039`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 3: user checkpoint, lines 92-92)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
