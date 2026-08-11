---
kind: workflow
name: standard-task-workflow
status: draft
version: 1
description: >-
  When the user brings a task, advance it through a fixed sequence of
  durable checkpoints — load context and select an execution mode, avoid
  duplicate queue entries, reconcile in-flight state, run a single
  dedicated validation boundary, then reconcile and deliver — never
  running cleanup ahead of verified terminal state and preserved evidence.
tags:
  - draft
  - icm-generated
---

# standard-task-workflow

Draft candidate materialized by `repo-to-icm`'s `60-draft` stage from
`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` bucket 1,
candidate 2. See `provenance.md` for the source `behavior_id`(s) and
`CONTEXT.md` for orientation. **Not runnable** — `status: draft`, lives
under `.sergeant/drafts/workflows/`, never `.sergeant/workflows/`
(`docs/icm/convention.md` §2).
