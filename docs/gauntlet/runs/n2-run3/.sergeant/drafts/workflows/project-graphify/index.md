---
kind: workflow
name: project-graphify
status: draft
version: 1
description: >-
  Trigger: sgt-graphify is invoked to extract and publish a project's
  knowledge graph. Outcome: publication is atomic-after-completion, never
  overlaps or destroys a source repo, and a failed or incomplete run is never
  promoted to the published location. Completion: publish-graph stops the run
  before publication if extraction produced zero matched repos, or any repo's
  extraction failed.
tags: []
---

# Project Graphify

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `project-graphify`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
