---
kind: workflow
name: worker-response-and-recovery
status: draft
version: 1
description: >-
  Trigger: a worker signals a nonterminal state (waiting/needs_input/blocked),
  or a wake condition becomes permanently unsatisfiable. Outcome: the worker
  is either resumed through a verified wake/response round-trip, or escalated
  to a human decision -- never guessed at or force-recovered. Completion: the
  response/resume action is durably recorded and the worker's own consumption
  of it completes the round-trip.
tags: []
---

# Worker Response And Recovery

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `worker-response-and-recovery`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
