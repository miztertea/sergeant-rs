---
kind: workflow
name: code-review
status: draft
version: 1
description: >-
  Trigger: both axes are ready to be evaluated.
  Outcome: the two axes stay visibly separate in the final report.
  Completion: Every member stage below has reached its own outcome, ending in stage `aggregate-review-report`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# code-review

Established by `BU-0927` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L6-11)`): A code review since a fixed point is evaluated along two independent axes: whether the code conforms to the repo's documented coding standards (Standards), and whether it faithfully implements what the originating issue/spec asked for (Spec).

**Trigger:** both axes are ready to be evaluated

**Outcome:** the two axes stay visibly separate in the final report

**Completion condition:** every member stage below has reached its own outcome, ending in stage `aggregate-review-report`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
