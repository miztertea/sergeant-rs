---
kind: workflow
name: list-projects
status: draft
version: 1
description: >-
  Trigger: the project-listing step is invoked.
  Outcome: only genuine project YAMLs are listed, and an empty result is reported explicitly rather than silently.
  Completion: The sole inferred stage `list-projects` has reached its outcome (design inference — see `provenance.md`).
tags:
  - draft
  - repo-to-icm-n2-run4
---

# list-projects

No `stage`-rung record in this run's corpus carries `workflow=list-projects` directly; this candidate's evidence is its workflow-level helpers alone (see `provenance.md`).

**Trigger:** the project-listing step is invoked

**Outcome:** only genuine project YAMLs are listed, and an empty result is reported explicitly rather than silently

**Completion condition:** the sole inferred stage `list-projects` has reached its outcome (design inference — see `provenance.md`).

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
