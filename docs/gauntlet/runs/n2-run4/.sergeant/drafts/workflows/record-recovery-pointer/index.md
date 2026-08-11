---
kind: workflow
name: record-recovery-pointer
status: draft
version: 1
description: >-
  Trigger: the task-tracker memory step is invoked with a worktree path.
  Outcome: git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test.
  Completion: Every member stage below has reached its own outcome, ending in stage `bind-worktree-identity`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# record-recovery-pointer

**Trigger:** the task-tracker memory step is invoked with a worktree path

**Outcome:** git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test

**Completion condition:** every member stage below has reached its own outcome, ending in stage `bind-worktree-identity`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
