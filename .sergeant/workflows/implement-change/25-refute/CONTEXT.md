# 25-refute: refute

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-panel/output/findings.md | L4 | the typed finding set this stage attacks |

## Purpose

Every raised finding carries a refuter verdict; findings not overturned
are `confirmed`, all others stay `refuted`.

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

## What must become true here (durable outcome)

Every finding in `output/findings.md` carries a final `status` —
`confirmed` only where a refuter seat attacked it and could not overturn
it, `refuted` in every other case — and, where `refuted`, the refuter's
own argument.

## Behavior contract

Apply `@@refute`. This package's own narrowing:

- **Every finding arrives `refuted` and stays `refuted` unless a refuter
  seat, having attacked it, reports that it could not overturn it and
  states what it checked.**
  (trigger: this stage begins; outcome: confirmation is something a finding
  must earn, never a status it inherits from being raised)
- **One refuter seat per axis, all spawned in a single message, each given
  only its own axis's findings, the pinned revision and diff command, and
  a 300-word cap. A refuter that can see another axis's findings is not
  spawned.**
  (trigger: spawning refuters; outcome: no axis's judgment can be
  laundered into another's, and each verdict is traceable to one axis)
- **A refuter seat's brief instructs it to attack, not to arbitrate:
  reproduce the defect, or show that it is wrong, stale, already handled
  elsewhere, or outside this change's scope — and to say which.**
  (trigger: writing the refuter brief; outcome: the seat's output is an
  argument with evidence, not a vote)
- **Silence, an empty report, a crashed seat, or an ambiguous verdict
  leaves every finding in that axis `refuted`. Absence of a refutation is
  never a confirmation.**
  (trigger: a refuter seat fails or hedges; outcome: the stage fails safe
  and `30-fix-confirmed` does less work rather than the wrong work)
- **Each finding's row is updated in place with its `status` and, when
  `refuted`, the `refutation` text. No finding is deleted from the set.**
  (trigger: recording verdicts; outcome: the set stays complete and a later
  reader can see what was raised and did not survive)
- **This stage does not fix anything and does not add new findings. A
  defect a refuter notices while attacking is recorded as a recommended
  follow-up, not appended to this panel's set.**
  (trigger: a refuter raises something new; outcome: the panel's set stays
  the panel's set, and scope does not broaden silently)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase each refuter's brief within the bounds above.

### J1 — local choices allowed
Exact invocation wording, so long as all refuters are spawned in one
message, each seeing only its own axis.

### J0 — must become `needs_input`
A refuter's verdict turns on a decision only the human can make (a scope
question, a policy question, an intentional-breaking-change question):
escalate with the finding, the refuter's argument, and the decision
required.

### Completion boundary
Every finding in the set has a `status` and, where `refuted`, a
`refutation`.

### Decision evidence
`output/findings.md`, updated in place.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
