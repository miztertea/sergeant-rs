# 30-parallel-review-spec: parallel review spec

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-parallel-review-standards/output/README.md | L4 | upstream artifact produced by `20-parallel-review-standards` |

## Purpose

An isolated review against the identified spec source.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

An isolated review against the identified spec source.

## Behavior contract

- **The Standards and Spec reviews run as parallel, isolated sub-agents so neither review's context pollutes the other, and this skill aggregates both sets of findings afterward.**
  (trigger: both review sources are identified; outcome: two independent review passes are produced and later merged into one report)
  — `BU-P2-003`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Process intro, lines 11-11)
- **The Standards sub-agent's prompt must include the diff command and commit list, the located standards-source files plus the full smell baseline pasted in (since the sub-agent has no other access to it), and a brief asking it to report per-file/hunk hard standard violations (cited to the standard) and baseline smells (named and quoted), distinguishing hard violations from judgment calls, skipping tooling-enforced items, under 400 words.**
  (trigger: spawning the Standards sub-agent; outcome: the sub-agent has everything needed to produce a bounded, well-formed Standards report)
  — `BU-P2-013`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 4: Standards sub-agent prompt, lines 62-66)
- **The Spec sub-agent's prompt must include the diff command and commit list, the path or fetched contents of the spec, and a brief asking it to report missing/partial requirements, scope creep (unasked-for behavior), and requirements that look implemented but wrong, quoting the spec line for each finding, under 400 words.**
  (trigger: spawning the Spec sub-agent; outcome: the sub-agent has everything needed to produce a bounded, well-formed Spec report)
  — `BU-P2-014`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 4: Spec sub-agent prompt, lines 68-72)
- **If no spec is found, the Spec sub-agent is skipped entirely and the final report notes this explicitly.**
  (trigger: no spec source was located in Step 2; outcome: the Spec axis is honestly reported as unavailable rather than fabricated)
  — `BU-P2-015`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 4: spec missing handling, lines 74-74)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
