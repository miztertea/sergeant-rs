---
name: review-outcome
description: Interpret something Sergeant returned — completed work, findings, blockers, conflicting evidence, or a suggested follow-up intent — against the intent's declared acceptance criteria, and state which disposition applies. Use when Sergeant returned something and it needs interpreting - accept, reject, revise, remediate, or redispatch.
edition: 0.2.0
---

Merged, adopted into the kernel.

## When to use

Sergeant returned something — completed work, findings, blockers,
conflicting evidence, or a suggested follow-up intent — and it needs
interpreting: accept / reject / revise / remediate / redispatch.

## The interactive protocol

Read the returned evidence against the intent's declared acceptance
criteria (per `define-acceptance`'s output, if the intent carried one).
State which of the five dispositions applies and why.

This is the **authorizing seat** for `remediate-findings`: a typed
finding set from `review-change` does not get remediated because a
workflow produced it — a human accepts it here first, matching
`AGENTS.md`'s PACE non-transfer of scope changes.

## Bounded judgment

### This skill may decide
- Which disposition (accept/reject/revise/remediate/redispatch) the
  returned evidence supports, when the evidence itself is unambiguous.

### This skill must ask the user
- Whenever the returned evidence is ambiguous about which disposition
  applies.
- Whenever authorizing `remediate-findings` would itself be a scope
  change.

### This skill must not do
- Treat a Work merely reaching a terminal state as evidence of
  correctness — "liveness is not evidence" generalizes directly:
  terminality is not correctness either.
- Authorize remediation on a finding set that isn't the typed shape
  `review-change`'s `40-report` stage actually emits.

### Durable handoff
The disposition decision itself, and — when the disposition is
`remediate-findings` — the human authorization that workflow's
`00-ingest` stage requires before it will accept a finding set at all.

## Failure behavior

If this invocation has no live human who will send the next message and
the returned evidence is ambiguous about which disposition applies, this
skill cannot resolve that ambiguity on its own. Say so plainly and leave
the disposition unresolved rather than guessing at accept/reject/revise/
remediate/redispatch.
