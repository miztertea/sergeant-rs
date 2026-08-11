---
kind: workflow
name: check-repo-status
status: draft
version: 1
description: >-
  Trigger: the status step is invoked for one or more repos.
  Outcome: each repo's status is reported from only verified, currently-observable git/filesystem state, never assumed.
  Completion: The sole inferred stage `report-repo-status` has reached its outcome (design inference — see `provenance.md`).
tags:
  - draft
  - repo-to-icm-n2-run4
---

# check-repo-status

No `stage`-rung record in this run's corpus carries `workflow=check-repo-status` directly; this candidate's evidence is its workflow-level helpers alone (see `provenance.md`).

**Trigger:** the status step is invoked for one or more repos

**Outcome:** each repo's status is reported from only verified, currently-observable git/filesystem state, never assumed

**Completion condition:** the sole inferred stage `report-repo-status` has reached its outcome (design inference — see `provenance.md`).

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
