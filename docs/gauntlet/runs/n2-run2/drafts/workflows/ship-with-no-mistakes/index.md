---
kind: workflow
name: ship-with-no-mistakes
status: draft
version: 1
description: >-
  When a shipping-gate run is about to be started, is active, or reaches an
  end state, launch it under strict invocation discipline and drive it
  under strict handling rules while active; on failure or cancellation,
  follow the reported recovery action exactly, and route actionable
  findings out to task-tracker tasks rather than fixing them inline.
tags:
  - draft
  - icm-generated
  - no-member-stages
---

# ship-with-no-mistakes

Draft candidate materialized by `repo-to-icm`'s `60-draft` stage from
`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` bucket 1,
candidate 3. See `provenance.md` for the source `behavior_id`(s) and
`CONTEXT.md` for orientation — including why this package, unlike its two
siblings, has **no `NN-<stage-name>/` directories at all**. **Not
runnable** — `status: draft`, lives under `.sergeant/drafts/workflows/`,
never `.sergeant/workflows/` (`docs/icm/convention.md` §2).
