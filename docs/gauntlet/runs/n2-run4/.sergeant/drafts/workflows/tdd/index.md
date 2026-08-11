---
kind: workflow
name: tdd
status: draft
version: 1
description: >-
  Trigger: a TDD cycle is about to begin and seams have not yet been agreed.
  Outcome: work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases.
  Completion: Every member stage below has reached its own outcome, ending in stage `run-red-green-loop`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# tdd

**Trigger:** a TDD cycle is about to begin and seams have not yet been agreed

**Outcome:** work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases

**Completion condition:** every member stage below has reached its own outcome, ending in stage `run-red-green-loop`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
