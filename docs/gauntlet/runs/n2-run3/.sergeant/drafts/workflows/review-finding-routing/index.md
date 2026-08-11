---
kind: workflow
name: review-finding-routing
status: draft
version: 1
description: >-
  Trigger: a dispatched worker submits a review-finding artifact to sgt-
  review-findings. Outcome: the finding is normalized, deduplicated, and
  routed into exactly one of four defined dispositions as an owning-repo td
  card, without ever silently overwriting a hand-edited card. Completion:
  route-finding, followed by reconcile-hand-edit on any rerun that meets a
  card modified outside the router.
tags: []
---

# Review Finding Routing

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `review-finding-routing`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
