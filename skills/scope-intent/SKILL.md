---
name: scope-intent
description: Establish where the work applies and where it does not — repository/artifact targets, components/interfaces, dependency boundaries, explicit non-goals, blast radius, single-repo vs. estate topology. Use once a precise outcome exists and needs its scope and boundaries drawn before it leaves the conversation.
edition: 0.2.0
---

Merged and load-bearing; absorbs `cross-repo-work`'s ownership/
dependency-order planning and, alongside `clarify-intent` and
`define-acceptance`, the "Implementation Decisions" / "Out of Scope" half
of `to-spec`'s retired template — the modules to be built/modified, their
interfaces, and boundaries are exactly this skill's own remit.

## When to use

Where the work applies and where it does not needs establishing —
repository/artifact targets, components/interfaces, dependency
boundaries, explicit non-goals, expected blast radius, and single-repo
vs. estate topology.

## The interactive protocol

For multi-repository work, produce the `targets: {repositories: [...],
dependency_order: [...]}` intent field — `cross-repo-work`'s ownership/
dependency-order judgment relocated from a dissolved workflow into
Captain-side, pre-dispatch, in-conversation scoping. The per-repository
fan-out that *consumes* this field is runtime, never this skill's job.

## Bounded judgment

### This skill may decide
- Which existing seam/boundary is the right scope cut when more than one
  is plausible, before putting the remaining ambiguity to the human.

### This skill must ask the user
- Explicit non-goals and dependency ordering whenever more than one
  plausible reading exists.
- Single-repo vs. estate topology whenever the intent doesn't obviously
  say.

### This skill must not do
- Restate the safety-sensitive keyword set — it has one home,
  `AGENTS.md`'s INTENT section; this skill points there rather than
  restating it, the same way `dispatch`'s retired `05-classify-risk`
  stage pointed rather than restated.
- Perform the per-repository fan-out itself — that is runtime, not this
  skill's job.

### Durable handoff
The `targets`/scope fields on the intent that eventually reaches `sgt
run`, or `select-workflow`'s selection record. Not a standalone repo
artifact of its own.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot ask the non-goal/dependency-order/topology questions its own
procedure depends on. Say so plainly and hold the scope as unconfirmed
rather than guessing at boundaries the human never actually drew.
