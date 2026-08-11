---
kind: workflow
name: sergeant-help
status: draft
version: 1
description: >-
  Trigger: sergeant-help is answering a question.
  Outcome: each condition triggers its own fixed required action rather than an ad hoc response.
  Completion: Every member stage below has reached its own outcome, ending in stage `handle-failure-or-handoff`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# sergeant-help

Established by `BU-0123` (`skills/sergeant-help/SKILL.md (skills/sergeant-help/SKILL.md L8-13)`): The sergeant-help skill is loaded for questions about what Sergeant is, install/configure/use, skill sources, running a command/workflow, or diagnosing an error, but is never loaded as a substitute for `load-project`, `cross-repo-work`, `dispatch`, or `wiki` once the user has requested execution of those procedures.

**Trigger:** sergeant-help is answering a question

**Outcome:** each condition triggers its own fixed required action rather than an ad hoc response

**Completion condition:** every member stage below has reached its own outcome, ending in stage `handle-failure-or-handoff`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
