# ADR 0007: The product states the actor's execution model

**Status:** Accepted, 2026-08-14.

## Context

An actor running a headless turn gets one turn and no callbacks: it cannot
background a long-running command and be woken when that command finishes.
Nothing in this repo's product surface stated that fact anywhere an actor
could read it before this decision. The owner's framing for why this
matters at all: "we deal in contracts, boundaries, and frameworks" — the
runtime model an actor operates under is exactly the kind of thing that
belongs in a contract rather than in an individual brief author's memory.

The cost of leaving it unstated was not hypothetical: actors guessed
wrong about their own execution model twice on 2026-08-14, and lost their
work both times (tracked as **issue #94**). The second occurrence's
transcript is explicit about the shape of the failure: the actor
considered a wakeup mechanism, correctly reasoned that it did not apply
to its situation, and then said "I'll simply wait for the background task
notification instead." That is not carelessness. It is a coherent
inference from a runtime nobody had described to it — the actor reasoned
its way to a wrong conclusion because the right one was never stated
anywhere for it to find.

The interview also records the orchestrating session's own fault in the
second occurrence, and that fault must stay part of the record rather than
being absorbed into "the actor guessed wrong": the failure was provoked by
a dispatch brief that demanded empirical proof — "run two m7 suites
concurrently and show they both pass" — which required long-running
background work to satisfy. A well-formed-looking acceptance criterion
steered the actor straight into the exact trap this ADR exists to close.
The brief author was not exempt from the problem it was diagnosing.

## Decision

**Two independent parts (D3).**

**(a) Whatever composes an actor's context states what wakes it.** Every
actor needs to know, alongside the environment guarantee ADR 0006
establishes, what its own runtime model actually permits: whether it can
background a command and be woken by a notification, or whether it gets
one turn with no callback and must plan accordingly. This is stated by the
product — the same context-composition step that fixes the environment
under ADR 0006 — not left to a brief author to remember to explain every
time.

**(b) A safety net for when an actor guesses wrong anyway.** Independently
of (a), a closing stage that declares a commit as its durable outcome must
not report plain `completed` when the branch never actually advanced and
the worktree is left dirty. This is the backstop for the case the contract
in (a) is meant to prevent but cannot guarantee against: an actor that
still misjudges its own runtime model should not have that misjudgment
silently reported as success.

## Alternatives considered

**Brief discipline instead of a product-level contract** — every dispatch
brief states the actor's execution model itself, as part of good brief
authorship, rather than the product stating it once — was raised as a
counter-argument and explicitly rejected by the owner. It is cheaper:
enforceable immediately, at zero implementation cost, by simply writing
better briefs. It was rejected anyway, because the product should state
its own contracts rather than depend on every future brief author
remembering to restate one, and because the failure mode when a brief
*doesn't* restate it is silent — nothing fails loudly when a brief omits
the execution-model statement, the actor simply guesses, sometimes wrong,
exactly as it did twice on 2026-08-14.

## Consequences

This is a product change that does not exist yet in this codebase. The ADR
records the two-part contract; it does not implement the context-
composition addition in (a) or the closing-stage guard in (b). Both remain
separate, not-yet-filed implementation work against issue #94.

Recording the orchestrator's own fault in the second #94 occurrence is
itself a consequence worth stating plainly: this decision is not only
"actors need better information," it is also "briefs that demand
empirical proof need to be written with this runtime model in mind,"
because a correctly-worded acceptance criterion produced the same failure
a missing contract would have produced on its own.

## Open questions

The exact mechanism for (a) — what the context-composition step says,
where an actor reads it, and whether it differs by backend or harness — is
not specified in the interview beyond "whatever composes an actor's
context states what wakes it." This ADR records the requirement, not its
implementation.

The exact detection logic for (b) — what distinguishes "worktree dirty
because the actor is still working" from "worktree dirty because the
actor guessed wrong and left work stranded," and what state a closing
stage should report instead of plain `completed` — is not specified here.
