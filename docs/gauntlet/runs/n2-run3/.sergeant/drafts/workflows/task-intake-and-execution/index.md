---
kind: workflow
name: task-intake-and-execution
status: draft
version: 1
description: >-
  Trigger: a task is brought to a Sergeant session. Outcome: the task reaches
  a durably recorded terminal/deliverable state through the mode-appropriate
  execution path, with evidence-preserving cleanup only after that state is
  verified. Completion: reconcile-and-deliver confirms terminal state and
  preserved evidence before any cleanup runs.
tags: []
---

# Task Intake And Execution

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `task-intake-and-execution`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
