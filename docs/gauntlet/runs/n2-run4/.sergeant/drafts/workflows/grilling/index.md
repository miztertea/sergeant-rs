---
kind: workflow
name: grilling
status: draft
version: 1
description: >-
  Trigger: a grilling interview is in progress.
  Outcome: the user is never presented with more than one open question at once.
  Completion: Every member stage below has reached its own outcome, ending in stage `conduct-interview`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# grilling

Established by `BU-0970` (`.agents/skills/grilling/SKILL.md (.agents/skills/grilling/SKILL.md L6-6)`): A grilling interview questions the user relentlessly about every aspect of the plan/decision/idea, walking each branch of the decision tree one by one and resolving dependencies between decisions, with a recommended answer offered for each question.

**Trigger:** a grilling interview is in progress

**Outcome:** the user is never presented with more than one open question at once

**Completion condition:** every member stage below has reached its own outcome, ending in stage `conduct-interview`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
