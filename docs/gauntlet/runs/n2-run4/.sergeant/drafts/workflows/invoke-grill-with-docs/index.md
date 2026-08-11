---
kind: workflow
name: invoke-grill-with-docs
status: draft
version: 1
description: >-
  Trigger: the grill-with-docs skill is invoked.
  Outcome: the resulting interview both stress-tests the plan and leaves behind ADR/glossary docs.
  Completion: The full six/five-part procedure named in the statement above has been executed, per `BU-0969`.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# invoke-grill-with-docs

**Single-behavior workflow candidate**, established entirely by `BU-0969`
(`.agents/skills/grill-with-docs/SKILL.md (.agents/skills/grill-with-docs/SKILL.md L7-7)`) — no `stage`/`stage-context`/`helper` record in this
corpus carries a `workflow` field matching `invoke-grill-with-docs` (checked against
that source file's other extracted units and against every `workflow`
field value used corpus-wide). Reported as a single-behavior candidate
per synthesis-method.md's "what must not happen" rather than padded out
or merged into an adjacent-topic cluster it does not actually name.

**Trigger:** the grill-with-docs skill is invoked

**Outcome:** the resulting interview both stress-tests the plan and leaves behind ADR/glossary docs

**Completion condition:** the full six/five-part procedure named in the statement above has been executed, per `BU-0969`.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
