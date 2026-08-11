---
kind: workflow
name: fleet-monitor-and-reconcile
status: draft
version: 1
description: >-
  Trigger: sgt-watch runs to snapshot fleet state (--snapshot), reconcile it
  in bulk (--sync-all), or assess a worker's health. Outcome:
  busy/health/notification state is reported strictly from verified evidence
  -- never fabricated as idle, healthy, or a known basis value when the
  verification conditions don't all hold. Completion: no stage in this corpus
  names it -- see provenance.md.
tags: []
---

# Fleet Monitor And Reconcile

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `fleet-monitor-and-reconcile`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
