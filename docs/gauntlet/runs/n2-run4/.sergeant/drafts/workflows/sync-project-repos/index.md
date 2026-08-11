---
kind: workflow
name: sync-project-repos
status: draft
version: 1
description: >-
  Trigger: the sync step runs against an already-cloned repo.
  Outcome: cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on.
  Completion: Every member stage below has reached its own outcome, ending in stage `clone-missing-repo`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# sync-project-repos

**Trigger:** the sync step runs against an already-cloned repo

**Outcome:** cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on

**Completion condition:** every member stage below has reached its own outcome, ending in stage `clone-missing-repo`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
