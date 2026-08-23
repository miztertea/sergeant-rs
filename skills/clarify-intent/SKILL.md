---
name: clarify-intent
description: Turn "I want this thing" or "this seems broken" into a precise outcome or problem statement — synthesized from this conversation and codebase exploration, never a fresh interview. Use when desired outcome needs distinguishing from proposed implementation, fact from assumption, requirement from preference, known constraint from unresolved question.
edition: 0.2.1
---

Merged; absorbs `to-spec`'s synthesis half (the "Problem Statement" /
"Solution" / "User Stories" material `to-spec`'s retired template used to
hold — §3 of this wave's spec covers the retirement).

## When to use

"I want this thing" or "this seems broken" needs to become a precise
outcome or problem statement.

## The interactive protocol

**Synthesize, do not interview.** This is the one D.1 skill that inherits
`to-spec`'s defining constraint rather than `grilling`'s: work from what
has already been said in this conversation plus this skill's own codebase
exploration, never a fresh round of questions — interviewing is
`grilling`'s job, named here so the two are never confused.

Once a precise outcome/problem statement is drafted, put it to the human
in one confirming exchange before treating it as settled — `to-spec`'s
own inherited constraint.

## Bounded judgment

### This skill may decide
- Whether existing conversational content is already sufficient to
  synthesize from, or more exploration is needed first.

### This skill must ask the user
- Nothing to *open* the procedure — it does not interview.
- One confirming exchange once a precise outcome/problem statement is
  drafted, before treating it as settled.

### This skill must not do
- Interview the user about the plan/design — that is `grilling`.
- Invent tracker or label vocabulary that does not exist in the estate —
  the same hazard `to-spec` guarded against: an upstream template
  assuming a taxonomy this estate never ported.
- Run via `sgt run` or any durable Work dispatch — the entire procedure
  depends on this conversation's own content, which a dispatched
  execution cannot receive.

### Durable handoff
A precise outcome/problem statement, consumed by
`scope-intent`/`define-acceptance`/`decide` in the same conversation or
carried into an intent composed for `sgt run`. Not itself a repo
artifact.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot perform the one confirming exchange its own procedure
requires. Say so plainly and hold the draft as unconfirmed rather than
treating it as settled.
