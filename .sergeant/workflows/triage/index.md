---
kind: workflow
name: triage
status: published
version: 3
description: >-
  Work through the attention queue: gather context, verify claims, recommend a disposition, and apply the terminal outcome with its required artifact.
tags:
  - triage
  - queue-management
---

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/triage.md` — this package's `BU-####`
citations and `reference/sergeant-upstream/` paths were stripped from the
shipped template content below; the record of why each rule exists did
not move with them.

# Triage

Five-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W30**) that works through the
attention queue: gathered context, verified claims, a recommended
disposition, an escalation to interview when underspecified, and the
terminal outcome applied with its required artifact. Use when: an item is
at the front of one of the three fixed attention buckets, oldest first.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the pinned
stage order. The full behavior-unit citation trail lives in
`docs/icm/promotion-spec-2026-08-11.md` and the archived
`docs/gauntlet/promoted-provenance/triage.md` (verbatim copy of this
package's pre-promotion `provenance.md`, plus a promotion note).
