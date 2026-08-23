---
name: select-workflow
description: Read `.sergeant/index.md` live and recommend a workflow, the applicable shared-context policies, and a delivery posture for an intent that's ready to leave the conversation — never restating the catalog or deciding for the human. Use when an intent is ready to leave the conversation and needs a workflow, policy, and delivery recommendation before `sgt run`.
edition: 0.2.0
---

Merged; the deliverable of ruling (b) — the `AGENTS.md` dispatch-time
discipline this skill's own output satisfies ("consult
`.sergeant/index.md` and name the workflow you selected... An unnamed
default is not a selection").

## When to use

An intent is ready to leave the conversation and needs a workflow,
policy, and delivery recommendation before `sgt run`.

## The interactive protocol

Read `.sergeant/index.md` live — never restate it. Recommend: the
workflow (one of the W2 seven named packages, or the embedded default
loop named honestly when nothing published fits — the embedded
`software-change` default is not retired and naming it is a valid,
citable outcome of this skill, not a failure of it); the applicable
shared-context policies for the work in view; and a delivery posture. The
human may override the recommendation. Produce a **dated selection
record** naming what was passed over and why, in this shape:

```
intent: change
workflow: software-change        # or: none — default loop
targets: [repos...]
policies: [test-first, independent-review, safety-sensitive]
delivery: { stop_at: validated-working-tree }
passed_over:
  - review-change: no diff exists yet
  - investigate: the question is already settled
```

## Bounded judgment

### This skill may decide
- How to phrase the recommendation and its rationale, provided the
  selection record names what was passed over and why.

### This skill must ask the user
- Nothing to open the procedure — it reads the live catalog and
  recommends. It asks only if the intent itself is ambiguous about which
  published workflow's remit it falls under, and defers to `decide` for
  a genuinely contested choice between close alternatives.

### This skill must not do
- Restate `.sergeant/index.md`'s catalog content instead of reading it
  live — the catalog is the source of truth, not a copy kept here.
- Decide for the human — the human may always override the
  recommendation; this skill recommends, it does not bind.
- Claim "no named workflow fits, use the embedded default" is a failure
  of this skill — it is a valid, honestly-stated outcome (§0's scope
  correction: the embedded `software-change` default is not retired).
- Restate the ROUTING table — dispatch-vs-in-session is `AGENTS.md`'s
  judgment and this skill applies it, never re-derives it.

### Durable handoff
The dated selection record — the workflow recommended, the policies
named, the delivery posture, and what was passed over and why — carried
into the `sgt run` submission that follows.

## Failure behavior

This skill's defining behavior (reading the live catalog and
recommending) does not itself require a live human turn — it can produce
a recommendation and selection record even without one, though a human
override never gets a chance to happen. State plainly that no
confirmation exchange occurred rather than implying the recommendation
was reviewed when it wasn't.
