---
kind: workflow
name: undocumented-failure-escalation
status: draft
version: 1
description: >-
  Trigger: a failure is not covered by existing documentation. Outcome:
  sergeant-help is used to search the docs, then the gap is escalated as a
  well-formed td task containing the exact reproduction, expected behavior,
  preserved state, and acceptance criteria -- rather than left unresolved or
  guessed at. Completion: the td task exists with all four required fields.
tags: []
---

# Undocumented Failure Escalation

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `undocumented-failure-escalation`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
