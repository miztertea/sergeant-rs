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

## Failure behavior

If the harness you're running under cannot actually pause mid-turn for a
human answer (measured true for this host's `claude` CLI's non-interactive
turns — see `docs/environments/cerberus.md`), a "grill" invocation degrades
to your best-guess autonomous answers with nothing to confirm against. Say
so plainly rather than presenting an unconfirmed guess as a reached shared
understanding — that silent degradation is exactly the failure mode this
package's retirement from the Work-dispatch surface was measured to avoid.
