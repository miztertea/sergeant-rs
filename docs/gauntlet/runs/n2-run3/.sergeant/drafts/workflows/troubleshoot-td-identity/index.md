---
kind: workflow
name: troubleshoot-td-identity
status: draft
version: 1
description: >-
  Trigger: the td executable resolved on PATH does not support the required
  flags. Outcome: PATH is corrected to the required implementation rather than
  building a wrapper around the wrong one, until td create --help shows the
  required description/JSON/working-directory options. Completion: td create
  --help shows the required description/JSON/working-directory options.
tags: []
---

# Troubleshoot Td Identity

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `troubleshoot-td-identity`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
