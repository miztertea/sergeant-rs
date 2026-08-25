# Refute

Resolved as `@@refute` from `.sergeant/common/contexts/refute.md` per
`.sergeant/common/contexts/icm-policy.md` §4. Shared stage context, two or more consumers:
`implement-change/25-refute`, `fix-defect/55-refute`,
`review-change/25-refute`, `investigate/25-challenge`.

Every finding a panel raises enters this stage `refuted`. It becomes
`confirmed` only by surviving an attack aimed at overturning it —
never by default, never by silence.

## Contract

- **Default refuted.** A raised finding arrives `status: refuted` and
  stays that way unless its refuter seat, having attacked it, reports that
  it could not overturn it and states what it checked.
- **One refuter seat per axis, one message.** Each sees only its own
  axis's findings — never another axis's — so no axis's judgment can be
  laundered into another's. A 300-word cap; refuters emit verdicts, not
  prose.
- **Attack, don't arbitrate.** A refuter's brief instructs it to
  reproduce the defect, or show it is wrong, stale, already handled
  elsewhere, or out of this change's scope — and to say which.
- **Silence never confirms.** An empty report, a crashed seat, or an
  ambiguous verdict leaves every finding in that axis `refuted`. Absence
  of a refutation is never treated as a confirmation.
- **The finding is updated in place**, `status` and, where `refuted`, the
  `refutation` text filled in. Nothing is deleted from the set.
- **This stage fixes nothing and raises nothing new.** A defect a refuter
  notices while attacking becomes a recorded recommended follow-up, never
  an addition to the panel's own set.

**What "isolated" means here, exactly.** These seats are sub-agents
spawned inside one stage execution, not separate Works. Their isolation
is context isolation only: each seat sees the brief this stage hands it
and nothing of its siblings' reasoning. It is weaker than the isolation a
separate Work would give — the seats share this stage's single execution,
its journal entry, its usage window and its failure; there is no per-seat
journal, no per-seat recovery, and a stage that dies takes every seat's
unwritten output with it. `.sergeant/common/contexts/icm-policy.md` §6.3 places review
independence in the execution boundary: this stage has one execution boundary, not four.
Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

Evidence for the posture: foundation's plan panel 17 raised / 6 confirmed;
codex 6 raised / 5 confirmed / 1 refuted — roughly a third of raised
findings do not survive attack, and a fixer acting on unrefuted findings
buys churn.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a refuter's verdict turns on a decision
  only a human can make (a scope question, a policy question, an
  intentional-breaking-change question) — escalate with the finding, the
  refuter's argument, and the decision required.
- **J2 the caller retains:** how to phrase each refuter's brief within
  these bounds.
- **J1 the caller retains:** exact invocation wording, so long as all
  refuters are spawned in one message, each seeing only its own axis.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
