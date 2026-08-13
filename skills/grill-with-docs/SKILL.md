---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also captures ADRs and a glossary as decisions land. Use when a plan or design needs interview-style stress-testing that should also produce durable domain artifacts.
---

Ported from `.sergeant/workflows/grill-with-docs` (N1 candidate W29), which
retires for the same reason as `grilling`: North Star ruling R-NS-6
dissolves the WORKFLOW-IF-E3 category this package was provisionally left in
(`docs/icm/retriage-2026-08-11.md`) — the interview it runs needs a live
human answer this session can hold, which a dispatched `sgt run` Work item
cannot.

## When to use

A plan or design needs the `grilling` interview *and* should leave behind
durable decision records (ADRs) and defined terminology (a glossary), not
just a confirmed shared understanding in the transcript.

## Procedure

1. Load the `grilling` skill and run its interview to completion in this
   session — one question at a time, facts looked up rather than asked,
   decisions put to the user, no action until explicit confirmation of
   shared understanding.
2. As each decision lands during the interview, capture it as you go — don't
   wait until the end to reconstruct decisions from memory:
   - **An ADR** (architecture/decision record): the decision made, the
     alternatives considered, and why this one was chosen. sergeant-rs has
     no standing ADR directory of its own yet; place new ones under
     `docs/adr/<slug>.md` (create the directory on first use) unless the
     user names an existing location, and say so explicitly rather than
     silently picking a spot.
   - **A glossary entry**: any domain term the interview coined or
     sharpened, with its definition, added to (or creating) `docs/glossary.md`
     in the same repo the plan/design belongs to.
   Method reference (frozen evidence, not runnable procedure in this repo):
   `reference/sergeant-upstream/.agents/skills/domain-modeling/SKILL.md`.
3. After the interview's confirmation gate is reached, review the captured
   ADRs/glossary entries with the user before treating them as final — the
   same explicit-confirmation discipline `grilling` applies to the plan
   itself applies to what got written down about it.

## Failure behavior

Same as `grilling`: if this host's harness cannot actually pause mid-turn
for a human answer, say so plainly rather than presenting unconfirmed
guesses — for both the plan and any ADR/glossary entries drawn from it — as
a reached shared understanding.
