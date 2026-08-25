# Bounded-Judgment Ladder — specialization contract

Ratified by owner ruling ICM-R0 decision 1. Re-homed
2026-08-17 (ADR 0014 decision 10, resolving the conflict between that
decision and `.sergeant/common/contexts/icm-policy.md` §3 rule 2's anti-duplication rule):
the rung definitions themselves (J5–J0) are canonical in `AGENTS.md`'s
AUTHORITY section, including PACE and succession of authority. This file
no longer restates them. It carries what only this file needs: how a
package narrows the ladder, the decision-evidence shape, the conflict
rule, and a worked example. Referenced as `@@bounded-judgment` from any
stage or skill's own `## Bounded judgment` section — no package copies
this text; each adds only its local specialization.

**Unresolved for the owner:** this re-homing is a material change to an
artifact ADR 0013 decision 1 ratified. See the commit message that made
this change for the explicit J0 flag; this file's content should be
treated provisional until ruled on.

## Purpose

> **What authority allows this actor to decide this material question
> without returning to a human or higher authority?**

Rung definitions, the conflict rule, and authority inheritance are all
canonical in `AGENTS.md`'s AUTHORITY section. Check J5 through J0 in order
there; cite the first rung that actually resolves the decision.

This file carries only what `AGENTS.md` should not: how a **stage or
skill** declares its own envelope, what a J0 escalation looks like on the
wire, and where decisions get recorded.

## Stage/skill specialization

A stage's `## Bounded judgment` section names its local narrowing: which
J2 decision classes it delegates, which choices are J1, what must become
`needs_input` at J0, its completion boundary, and where decisions are
recorded — present even when it is only "inherits workflow envelope
unchanged" (omission is never ambiguous).

Canonical shape for a J0 landing:

```markdown
## Decision required — J0

**Decision:** May the public response schema change?
**Checked:** J5 no policy grants a breaking change; J4 acceptance requires
compatibility; J3 no accepted migration record exists; J2 this stage may
propose designs but may not alter public behavior; J1 does not apply.
**Evidence:** ...
**Recommendation:** Preserve the existing schema and add an optional field.
**Question:** Should this Work preserve backward compatibility, or may it
make an intentional breaking API change?
```

For a workflow stage: record the unresolved decision, state which rungs
were checked and why they didn't settle it, preserve the evidence already
gathered, state a recommendation when one can be responsibly offered, and
end the turn with one direct question so the existing backend signal
places the Work in `needs_input`. For a Captain skill: ask the question
live and wait for the user's answer before continuing.

## Decision evidence

Every package defines where material decisions are recorded. Recommended
default:

```markdown
| Decision | Rung | Evidence | Resolution |
|---|---|---|---|
| ... | J2 | ... | ... |
```

The table may live in a declared Layer-4 artifact, review report,
proposal, or final summary — the requirement is traceability, not one
universal filename.

## Worked example, from this ladder's own first live use

A Captain session (2026-08-16, this file's own ratification session) was
asked whether to dispatch all nine ICM-R2 pilot packages at once or pilot
a smaller slice first. **J5:** no policy forbids parallel dispatch; the
daemon's own turn/ceiling envelope already bounds per-Work cost, so there
is no unbounded risk requiring escalation. **J4:** the owner's own prior
decisions in the same session ("go for it" on testing all three
controversial packages now; "a PR ready for review implementing this end
to end") already set the scope — asking again would be reconfirming a
decision already made, not resolving a new one. **Conclusion: J1** — batch
size and dispatch sequencing are local, reversible execution choices
within already-granted authority, not a scope change. The session had
initially asked the user anyway; the owner's correction ("those ladders
are there to help you know when to come to me") is the concrete evidence
that this was over-escalation.
