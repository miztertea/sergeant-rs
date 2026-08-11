# 00-agree-seams: agree seams

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Seams are written down and confirmed with the user; no test is written at an unconfirmed seam.

Trigger (workflow-level): A feature or bug fix is being implemented test-first.

## What must become true here (durable outcome)

Seams are written down and confirmed with the user; no test is written at an unconfirmed seam.

## Behavior contract

- **Test seams must be pre-agreed: before writing any test the actor writes down the seams under test and confirms them with the user, and no test is written at an unconfirmed seam, since agreeing seams up front is how testing effort lands on critical paths and complex logic instead of every edge case.**
  (trigger: about to write the first test of a cycle; outcome: the seam is explicitly confirmed with the user before any test is written)
  — `BU-P2-109`, `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Seams, lines 22-22)
- **The actor asks the user: 'What's the public interface, and which seams should we test?'**
  (trigger: confirming seams with the user; outcome: a concrete, repeatable question is used to elicit the seam agreement)
  — `BU-P2-110`, `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Seams, lines 24-24)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
