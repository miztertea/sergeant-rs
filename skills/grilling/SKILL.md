---
name: grilling
description: Grill the user relentlessly about a plan, decision, or idea, one question at a time. Use when the user wants to stress-test their thinking, or uses a 'grill' trigger phrase.
---

Ported from `.sergeant/workflows/grilling` (N1 candidate W28), which
retires: North Star ruling R-NS-6 ("execution ≠ dialogue") holds that
conversation is the harness's job, never engine work, and dissolves the
WORKFLOW-IF-E3 category the retriage classifier had provisionally left this
package in (`docs/icm/retriage-2026-08-11.md`). The dogfood measurement that
forced the call: on this host, both of the retired workflow's stages
completed autonomously with **zero** `needs_input` pauses in 2/2 runs — "negative
value vs plain terminal Claude" — because a durable `sgt run` stage has no
mid-turn hold for a human's answer to land in (`AGENTS.md`'s routing table
carried this exact caveat before this re-home). A live back-and-forth
belongs in this session, not in a dispatched Work item.

## When to use

The user wants their plan, decision, or idea stress-tested, or invokes a
"grill" trigger phrase. Run this interview directly in the current
conversation — never via `sgt run`.

## How to grill

Interview the user relentlessly about every aspect of the plan/decision/idea
until you reach a shared understanding. Walk down each branch of the
decision tree, resolving dependencies between decisions one at a time. For
each question, offer your own recommended answer alongside it.

- **One question at a time.** Ask, then wait for the user's answer before
  asking the next. Asking multiple questions at once is bewildering.
- **Facts vs. decisions.** If a fact can be found by exploring the
  environment (filesystem, `sgt doctor`, tests, docs, `--help` output, any
  tool available to you), look it up yourself rather than asking the user.
  The *decisions* are the user's — put each one to them and wait for their
  answer.
- **Do not act on the plan** until the user explicitly confirms shared
  understanding has been reached. That confirmation is a hard gate before
  any implementation, `sgt run` submission, or file edit driven by the
  interview's conclusions.

## Bounded judgment

Apply `@@bounded-judgment`.

### This skill may decide
- Which branch of the decision tree to walk next, and in what order,
  provided every dependent decision is resolved before decisions that
  depend on it.
- Whether a given fact is discoverable by exploring the environment
  (filesystem, `sgt doctor`, tests, docs, `--help`, any available tool)
  rather than a genuine decision that must go to the user.
- What recommended answer to offer alongside each question.

### This skill must ask the user
- Every genuine decision identified by the interview, one at a time,
  waiting for the answer before asking the next.
- Explicit confirmation that shared understanding has been reached,
  before acting on any of the interview's conclusions.

### This skill must not do
- Run via `sgt run` or any durable Work dispatch — R-NS-6 places this
  entirely inside the current conversation.
- Ask more than one question at a time.
- Act on the plan/decision/idea — implementation, `sgt run` submission,
  or a file edit driven by the interview's conclusions — before the
  user's explicit confirmation.
- Present an unconfirmed, harness-degraded best guess as a reached shared
  understanding; say so plainly instead.

### Durable handoff
None. This skill produces no promotable artifact of its own; a confirmed
understanding is consumed directly in the same session (e.g. to shape a
subsequent `sgt run` submission), not written to a Work surface.

## Failure behavior

If this invocation has no live human who will send the next message — the
harness is running headless/unattended, not mid-conversation with a
person — a "grill" invocation degrades to your best-guess autonomous
answers with nothing to confirm against. (Corrected 2026-08-16, ICM-R2
pilot review: the prior text here cited `docs/environments/cerberus.md`
for this claim, but that file measures a different mechanism entirely —
the engine's `post_turn_summary`/`NeedsInput` signal for a dispatched
workflow stage, not Captain-session turn-taking, which never goes through
that machinery. The real distinguishing fact is self-evident from the
harness's own execution mode and needs no engine-mechanics citation.) Say
so plainly rather than presenting an unconfirmed guess as a reached shared
understanding — that silent degradation is exactly the failure mode this
package's retirement from the Work-dispatch surface was measured to avoid.
