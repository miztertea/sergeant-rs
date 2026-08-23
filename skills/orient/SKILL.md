---
name: orient
description: Establish shared terminology, actors, and the immediate decision in view before any other skill or `sgt run` touches this system this session. Use when a session opens on a project/system/estate the human and Captain have not yet established a shared model for.
edition: 0.2.1
---

**[external]** — new authorship, no estate evidence behind it. This skill
is proposed on the argument that a fourteen-wide Captain kernel needs an
explicit seam-setting step before any other skill runs; the owner may
reject this skill later if the argument doesn't hold up in practice.

## When to use

A session opens on a project, system, or estate the human and Captain
have not yet established shared terminology, actors, or the immediate
decision for — before any other skill or `sgt run` in this session
touches it.

## The interactive protocol

Ask what system or decision is actually in view. Confirm shared
vocabulary for the actors and components named, one question at a time,
waiting for each answer, until the human's and Captain's models agree.

This skill does not re-run `estate-navigation`'s mechanical checks (`sgt
doctor`, `sgt repo list`, `sgt group list`) — the seam between the two is
explicit: `orient` answers "are we discussing the same system,"
`estate-navigation` answers "which repos exist, are they declared, are
they synced" mechanically. Load `estate-navigation` separately when the
mechanical question is the open one.

## Bounded judgment

### This skill may decide
- How many clarifying turns the shared-model check needs before moving
  on.

### This skill must ask the user
- Whenever the human's and Captain's models of the system/decision
  disagree, one question at a time until they converge.

### This skill must not do
- Perform `estate-navigation`'s repo/group/health checks itself —
  that is duplicate ownership of a mechanical question this skill does
  not answer.
- Treat "we both used the same nouns" as sufficient without confirming
  the immediate decision in view.
- Proceed to `brainstorm`/`clarify-intent` before the model check closes.

### Durable handoff
None — the shared model is consumed in-session by whatever skill runs
next.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot perform its defining behavior: there is no one to confirm a
shared model against. Say so plainly and stop rather than assuming
agreement that was never actually reached.
