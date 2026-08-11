---
kind: workflow
name: installation-and-setup
status: draft
version: 1
description: >-
  Trigger: installation, or mise run check/install/update, is invoked.
  Outcome: dependencies are verified against their real capability surface,
  and symlinks/hooks are (re)installed or removed idempotently, before
  Sergeant is considered usable. Completion: dependency-check passes for every
  required dependency.
tags: []
---

# Installation And Setup

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `installation-and-setup`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
