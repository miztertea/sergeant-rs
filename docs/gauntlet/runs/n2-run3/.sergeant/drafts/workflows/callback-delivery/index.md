---
kind: workflow
name: callback-delivery
status: draft
version: 1
description: >-
  Trigger: a callback event is registered, enqueued, or delivered to an
  external consumer. Outcome: origin identity, idempotency, bounded consumer
  execution, a closed outcome set, and requeue behavior are all deterministic.
  Completion: no stage checkpoint was classified in this corpus for this
  workflow -- see provenance.md.
tags: []
---

# Callback Delivery

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `callback-delivery`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
