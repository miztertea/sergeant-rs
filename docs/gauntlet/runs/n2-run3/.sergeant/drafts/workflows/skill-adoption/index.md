---
kind: workflow
name: skill-adoption
status: draft
version: 1
description: >-
  Trigger: an external skill is being adopted. Outcome: the six-step vetting
  procedure (read SKILL.md and referenced scripts; confirm source/update
  mechanism; check filesystem/shell/network/Git/credential actions; verify no
  conflict with AGENTS.md/safety policy; pin/lock the source where supported;
  test in a disposable repo/worktree) is completed before broad installation.
  Completion: all six checks done.
tags: []
---

# Skill Adoption

Draft workflow candidate materialized by `repo-to-icm`'s `60-draft` stage
from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
(candidate cluster `skill-adoption`). `status: draft` -- not promoted. Promotion to
`.sergeant/workflows/` is a human review decision
(`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
