---
kind: workflow
name: callback-protocol
status: draft
version: 1
description: >-
  Trigger: a callback profile is installed or invoked.
  Outcome: delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery.
  Completion: Every member stage below has reached its own outcome, ending in stage `retry-delivery`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# callback-protocol

**Trigger:** a callback profile is installed or invoked

**Outcome:** delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery

**Completion condition:** every member stage below has reached its own outcome, ending in stage `retry-delivery`'s.

**Ordering:** `callback-protocol` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
