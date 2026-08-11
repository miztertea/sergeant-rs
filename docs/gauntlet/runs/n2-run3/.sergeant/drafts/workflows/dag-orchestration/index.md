---
kind: workflow
name: dag-orchestration
status: draft
version: 1
description: >-
  Trigger: a DAG stage declares an after: dependency. Outcome: the stage
  becomes ready to dispatch only once its named predecessor stages have
  completed. Completion: stage-dependency-gate, advanced automatically by sgt-
  watch.
tags: []
---

# Dag Orchestration

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `dag-orchestration`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
