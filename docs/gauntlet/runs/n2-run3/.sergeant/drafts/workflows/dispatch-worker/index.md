---
kind: workflow
name: dispatch-worker
status: draft
version: 1
description: >-
  Trigger: dispatch mode has been selected for a task and work has been
  decomposed by owning repository. Outcome: each owning repository ends up
  with a durably launched, evidence-backed worker running under one stable
  canonical intent, or dispatch fails closed before mutating anything.
  Completion: every target repo either has a spawned worker with recorded
  launch evidence and a generation-tracked gate identity, or the dispatch
  aborted with no partial state left behind.
tags: []
---

# Dispatch Worker

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `dispatch-worker`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
