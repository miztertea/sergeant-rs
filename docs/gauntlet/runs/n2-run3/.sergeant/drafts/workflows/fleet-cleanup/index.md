---
kind: workflow
name: fleet-cleanup
status: draft
version: 1
description: >-
  Trigger: sgt-cleanup is invoked for a task. Outcome: cleanup proceeds only
  once every named precondition holds -- terminal proof, staged evidence, a
  converged or explicitly retired response handshake, and (when applicable)
  callback completion -- never as a shortcut for a nonterminal worker state.
  Completion: cleanup-preconditions.
tags: []
---

# Fleet Cleanup

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `fleet-cleanup`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
