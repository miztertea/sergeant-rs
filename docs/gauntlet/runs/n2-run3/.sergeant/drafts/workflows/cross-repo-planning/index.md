---
kind: workflow
name: cross-repo-planning
status: draft
version: 1
description: >-
  Trigger: a requested outcome is being decomposed across repositories.
  Outcome: exactly one repository is named as owning each required behavior,
  and a repository is included only when it must actually change or produce
  delivery evidence. Completion: ownership assignment for every required
  behavior.
tags: []
---

# Cross Repo Planning

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `cross-repo-planning`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
