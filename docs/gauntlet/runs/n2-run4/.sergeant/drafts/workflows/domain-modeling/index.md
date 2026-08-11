---
kind: workflow
name: domain-modeling
status: draft
version: 1
description: >-
  Trigger: the user uses a term that conflicts with CONTEXT.md's existing definition.
  Outcome: an ADR is offered only when the three-part test passes; otherwise it is not created.
  Completion: Every member stage below has reached its own outcome, ending in stage `offer-adr`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# domain-modeling

**Trigger:** the user uses a term that conflicts with CONTEXT.md's existing definition

**Outcome:** an ADR is offered only when the three-part test passes; otherwise it is not created

**Completion condition:** every member stage below has reached its own outcome, ending in stage `offer-adr`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
