---
kind: workflow
name: troubleshoot-failure
status: draft
version: 1
description: >-
  Trigger: a failure is not covered by existing documentation.
  Outcome: the gap is escalated as a well-formed task tracker task rather than left unresolved or guessed at.
  Completion: Every member stage below has reached its own outcome, ending in stage `escalate-undocumented-gap`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# troubleshoot-failure

**Trigger:** a failure is not covered by existing documentation

**Outcome:** the gap is escalated as a well-formed task tracker task rather than left unresolved or guessed at

**Completion condition:** every member stage below has reached its own outcome, ending in stage `escalate-undocumented-gap`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
