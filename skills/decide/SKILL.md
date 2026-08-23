---
name: decide
description: Compare alternatives and make an explicit decision with the human — options, comparison dimensions, recommendation, human decision, rationale, rejected alternatives and why, and when the decision should be revisited. Use when alternatives need comparing and the human needs to make an explicit decision.
edition: 0.2.0
---

Merged; carries J0's one-question discipline and absorbs the
never-shipped `adjudicate` skill.

## When to use

Alternatives need comparing and the human needs to make an explicit
decision.

## The interactive protocol

**The absorption, specified exactly.** `adjudicate` was never shipped;
its whole proposed content was `AGENTS.md`'s own J0 procedure, ruled but
never packaged as a skill: **do not guess — record the decision, state
which rungs were checked and why they didn't settle it, preserve the
evidence gathered, offer a recommendation when one can be responsibly
made, and end the turn with one direct question.** This skill performs
exactly that procedure, live, sharpened by two things a plain "compare
options" skill would not carry: the **one-question-per-turn** rule (J0's
own text), and PACE's **non-transferable decision classes** — destructive
or irreversible action, merges to a default branch, scope or policy
changes — which this skill must recognize and refuse to decide on the
human's behalf even when asked to.

## Bounded judgment

### This skill may decide
- Which comparison dimensions are relevant to surface for a given
  decision.
- How to phrase the recommendation.

### This skill must ask the user
- The one question that actually resolves a J0-shaped decision, after
  stating rungs checked and evidence — never more than one question in
  the same turn.

### This skill must not do
- Rule on the owner's behalf, or record a recommendation as if it were a
  ruling.
- Batch questions — J0 says one.
- Transfer a decision from a class PACE marks non-transferable
  (destructive or irreversible action, merges to a default branch, scope
  or policy changes), regardless of who asks.
- Run via `sgt run` or any durable Work dispatch — this skill's defining
  behavior is a live decision turn.
- Present an unconfirmed, harness-degraded best guess as a ruling; say so
  plainly instead.

### Durable handoff
This workspace's ratified `owner-ruling` record type
(`knowledge/TYPES.md`) under `knowledge/rulings/owner-rulings/` — a
workspace-side artifact, filled by hand today; this skill's decision
output is exactly that record's shape. (Workspace-side, not a
`sergeant-rs` repo write.)

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot ask the one question its own procedure ends on. Say so
plainly, record the rungs checked and evidence gathered, and stop short
of a ruling rather than guessing at one.
