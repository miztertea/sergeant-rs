---
name: decompose
description: Split a large, already-approved intent into independently executable child intents, with dependencies and sequencing between them. Use when a large approved intent needs splitting into units of work before each goes to select-workflow.
edition: 0.2.1
---

**[external]** — new authorship, no estate evidence behind it, same
caveat as `orient`/`brainstorm`.

## When to use

A large, already-approved intent needs splitting into independently
executable child intents, with dependencies and sequencing — deciding
*what units of work should exist*, never file-by-file instructions (a
workflow's job, not this skill's).

## The interactive protocol

Propose a decomposition. State dependencies and sequencing between the
child intents. Confirm with the human before treating any child intent as
ready for `select-workflow`.

## Bounded judgment

### This skill may decide
- The shape of a decomposition when multiple reasonable cuts exist,
  before presenting it.

### This skill must ask the user
- Confirmation of the decomposition and its sequencing before any child
  intent is treated as approved.

### This skill must not do
- Write per-file implementation instructions — that is a workflow's
  stage content.
- Dispatch any child intent itself — that is `select-workflow`'s and
  ultimately `sgt run`'s job, downstream of this skill's output.

### Durable handoff
The set of child intents and their declared dependency order, carried
into `select-workflow` per child.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot get the confirmation its own procedure requires before any
child intent is treated as ready. Say so plainly and hold every child
intent as proposed, not approved.
