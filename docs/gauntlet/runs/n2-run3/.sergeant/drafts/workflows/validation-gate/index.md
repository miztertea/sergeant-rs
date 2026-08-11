---
kind: workflow
name: validation-gate
status: draft
version: 1
description: >-
  Trigger: dispatched (or direct-mode) work reaches readiness for shipping
  validation. Outcome: exactly one validation launch runs to completion under
  a coordinator-verified, auditable transport, and readiness is durably
  published only once every gate has genuinely passed. Completion: readiness
  evidence anchored to a real, committed HEAD is recorded before the
  coordinator is notified.
tags: []
---

# Validation Gate

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `validation-gate`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
