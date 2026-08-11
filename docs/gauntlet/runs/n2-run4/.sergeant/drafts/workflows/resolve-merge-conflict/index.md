---
kind: workflow
name: resolve-merge-conflict
status: draft
version: 1
description: >-
  Trigger: the resolving-merge-conflicts skill is invoked.
  Outcome: the merge/rebase always reaches a resolved state rather than being abandoned mid-way.
  Completion: Every member stage below has reached its own outcome, ending in stage `complete-merge`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# resolve-merge-conflict

**Trigger:** the resolving-merge-conflicts skill is invoked

**Outcome:** the merge/rebase always reaches a resolved state rather than being abandoned mid-way

**Completion condition:** every member stage below has reached its own outcome, ending in stage `complete-merge`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
