---
kind: workflow
name: dispatch-mode
status: draft
version: 1
description: >-
  When work spans repositories, has independent repo-owned sub-tasks, needs
  isolated review, or the user explicitly requests workers, dispatch one
  worker per owning repository into an isolated checkout with a written
  brief and a spawned interactive agent session, then monitor progress
  through to reconciliation of merge order and cross-repo implications.
tags:
  - draft
  - icm-generated
---

# dispatch-mode

Draft candidate materialized by `repo-to-icm`'s `60-draft` stage from
`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` bucket 1,
candidate 1. See `provenance.md` for the source `behavior_id`(s) and
`CONTEXT.md` for orientation. **Not runnable** — `status: draft`, lives
under `.sergeant/drafts/workflows/`, never `.sergeant/workflows/`
(`docs/icm/convention.md` §2).
