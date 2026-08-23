---
name: brainstorm
description: Explore goals, approaches, constraints, non-goals, and consequences before an intent is precise enough to hand to clarify-intent. Use when what's wanted isn't yet settled enough to state as a candidate intent.
edition: 0.2.0
---

**[external]** — new authorship, no estate evidence behind it. Carried
under the same honest caveat as `orient`: this skill is proposed on the
argument that ambiguity resolution needs a pre-`clarify-intent` widening
step, and the owner may reject it later.

## When to use

Goals, approaches, constraints, non-goals, or consequences are not yet
settled enough to hand to `clarify-intent`.

## The interactive protocol

Explore possibilities without committing. Surface tradeoffs and
consequences across the branches that come up. Narrow toward — at most —
a **candidate** intent, never an implementation plan or code: this
skill's whole job is widening the space before `clarify-intent` narrows
it to a precise outcome statement.

## Bounded judgment

### This skill may decide
- Which explored branches are worth surfacing to the human versus
  silently dead-ended — a J1-shaped editorial call, reversible, scoped to
  this conversation.

### This skill must ask the user
- Which of several genuinely live candidate directions the human wants
  to carry forward, before narrowing further.

### This skill must not do
- Produce implementation detail, code, or a file edit.
- Present a narrowed candidate as a decided intent — that handoff belongs
  to `clarify-intent`/`decide`.
- Run via `sgt run` or any durable Work dispatch — this skill's defining
  behavior is a live, exploratory turn this session holds.
- Present an unconfirmed, harness-degraded best guess as a reached
  candidate direction; say so plainly instead.

### Durable handoff
None — output is narrowed possibilities or one candidate intent, carried
into `clarify-intent` in the same conversation.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot ask which candidate direction to carry forward. Say so
plainly rather than silently picking one and presenting it as the
human's choice.
