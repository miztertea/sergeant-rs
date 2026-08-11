---
kind: workflow
name: sergeant-help-query
status: draft
version: 1
description: >-
  Trigger: sergeant-help is answering a question. Outcome: the answer follows
  a fixed research sequence -- classify the question against the documentation
  map, read the primary document first, escalate to a repository-wide grep
  only for unresolved terms, consult graphify query for architectural
  questions when a graph exists -- rather than free-form search. Completion:
  the answer cites the exact command, required preconditions, expected
  evidence, and documentation links.
tags: []
---

# Sergeant Help Query

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `sergeant-help-query`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
