---
name: retrospective
description: Discuss what happened, what was learned, and whether a process, workflow, policy, or product change should become a new intent. Use when discussing the conversation around a postmortem's findings, at sprint scale or smaller.
edition: 0.2.0
---

**[proven]** — kept.

## When to use

Discuss what happened, what was learned, and whether a process, workflow,
policy, or product change should become a new intent — the conversation
around a postmortem's findings. A postmortem *workflow*, evidence-heavy
and reconstructing a timeline from artifacts, is a separate, future
package; this skill is the in-session discussion, not that.

## The interactive protocol

The same four-section shape `plan-sprint`'s retros use — `## Timeline` /
`## What the gauntlet caught that would have shipped` / `## Method
notes` / `## Owner follow-ups` — is the right default shape for a
sprint-scale retrospective. For a smaller retro, at minimum cover what
happened, what was learned, and what (if anything) should become a new
intent.

## Bounded judgment

### This skill may decide
- How much of the fixed four-section shape a given retrospective's scale
  actually needs.

### This skill must ask the user
- Whether any lesson surfaced should become a new intent, before
  treating the retrospective as closed.

### This skill must not do
- Silently promote a lesson into a new intent without the human naming
  it one.
- Fabricate a timeline from memory when the actual artifacts (journal,
  PRs, panel results) are available to read instead.
- Run via `sgt run` or any durable Work dispatch — this skill's defining
  behavior is a live discussion this session holds.
- Present an unconfirmed, harness-degraded best guess as a closed
  retrospective; say so plainly instead.

### Durable handoff
The retrospective document, when the human wants one kept; otherwise
None — a purely conversational retro is legitimate too.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot ask whether a surfaced lesson should become a new intent.
Say so plainly and leave the retrospective open rather than closing it
or silently promoting a lesson on the human's behalf.
