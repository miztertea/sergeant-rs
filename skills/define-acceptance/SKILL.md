---
name: define-acceptance
description: Turn subjective completion language ("done," "working," "fixed") into observable success — behavioral acceptance criteria, quality constraints, required tests/evidence, expected documentation, thresholds, and blocking conditions. Use once scope is drawn and acceptance still needs stating before an intent leaves the conversation.
edition: 0.2.1
---

Merged; absorbs `validate-intent` as an in-dialogue review.

## When to use

Subjective completion language ("done," "working," "fixed") needs to
become observable success.

## The interactive protocol

**The absorption, specified exactly.** `validate-intent` shipped as a
one-stage dispatched workflow that called itself "optional Captain
tooling" while running as a Work item — a self-contradiction this skill
closes by folding its check into a **live, in-dialogue review**, not a
separate dispatch: after drafting acceptance criteria, walk them against
`AGENTS.md`'s INTENT eight-dimension list (Objective, Required
Invariants, Approved Tradeoffs, Out Of Scope, State Transitions, Failure
Windows, Negative Test Matrix, Validation Evidence) **by pointer, never
by restating the list**, naming out loud which dimensions the drafted
criteria cover and which they don't, before treating the acceptance
criteria as final.

**Cost, stated honestly.** This trades away independence. A separate
dispatched seat checking Captain's own drafted brief is exactly the
blind-review property the sprint method values; folding the check into
the skill that drafted the criteria makes it a self-check.
**Mitigation:** spec-fidelity is a panel axis in both `implement-change`
and `review-change`, so the check survives downstream — one stage later
than a live `validate-intent` dispatch would have caught it, never
dropped.

## Bounded judgment

### This skill may decide
- How to phrase and organize the acceptance criteria, provided every one
  of the eight dimensions is checked against and named as covered or not.

### This skill must ask the user
- One at a time, wherever the dimension walk surfaces an uncovered
  dimension that needs a decision to close (a threshold, a required test
  class, a blocking condition).

### This skill must not do
- Restate the eight dimensions — point at `AGENTS.md`'s INTENT section
  instead.
- Claim its own in-dialogue self-check substitutes for independent
  review — state the cost plainly instead, per the paragraph above.
- Treat a Work merely completing as evidence of correctness — the
  "'tests passed' is not completion" evidence-requirement policy, homed
  at `.sergeant/common/contexts/evidence-requirements.md`, is named by
  pointer here, not restated.

### Durable handoff
Drafted acceptance criteria on the intent that reaches `sgt run`, or a
standalone acceptance record if the human wants one kept independent of a
specific dispatch. Not a Work-branch file — this skill never dispatches.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot ask about the dimensions the walk surfaces as uncovered. Say
so plainly and hold the criteria as a draft rather than treating an
unconfirmed set as final.
