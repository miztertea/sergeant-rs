---
kind: workflow
name: ci-verification
status: draft
version: 1
description: >-
  Trigger: the toolchain/task runner run test:docker:drain runs.
  Outcome: compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one.
  Completion: Every member stage below has reached its own outcome, ending in stage `verify-bash-compat-both-passes`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# ci-verification

**Trigger:** the toolchain/task runner run test:docker:drain runs

**Outcome:** compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one

**Completion condition:** every member stage below has reached its own outcome, ending in stage `verify-bash-compat-both-passes`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
