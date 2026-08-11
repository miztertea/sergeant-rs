---
kind: workflow
name: treehouse-init
status: draft
version: 1
description: >-
  Trigger: treehouse initialization is run for a repo.
  Outcome: initialization is idempotent: an already-initialized repo is reported as such rather than re-initialized.
  Completion: The sole inferred stage `initialize-treehouse` has reached its outcome (design inference — see `provenance.md`).
tags:
  - draft
  - repo-to-icm-n2-run4
---

# treehouse-init

No `stage`-rung record in this run's corpus carries `workflow=treehouse-init` directly; this candidate's evidence is its workflow-level helpers alone (see `provenance.md`).

**Trigger:** treehouse initialization is run for a repo

**Outcome:** initialization is idempotent: an already-initialized repo is reported as such rather than re-initialized

**Completion condition:** the sole inferred stage `initialize-treehouse` has reached its outcome (design inference — see `provenance.md`).

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
