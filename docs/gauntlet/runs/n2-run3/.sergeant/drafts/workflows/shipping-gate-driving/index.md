---
kind: workflow
name: shipping-gate-driving
status: draft
version: 1
description: >-
  Trigger: the coordinator drives a no-mistakes shipping gate to completion
  for dispatched (or direct-mode) work. Outcome: the gate is started at most
  once per precondition-satisfied run, polled rather than re-issued, and
  findings are routed by disposition. Completion: group-remediation converges
  remediation to one worker per shared root cause, rechecked before merge,
  escalating to a human after two unsuccessful cycles.
tags: []
---

# Shipping Gate Driving

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `shipping-gate-driving`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
