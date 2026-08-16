# Bounded-Judgment Ladder

Canonical source. Ratified by `docs/adr/0013-icm-r0-owner-rulings.md`
decision 1, per `reference/proposal-icm-r-procedure-authority.md` §6.
Referenced by `@@bounded-judgment` from any stage or skill's own
`## Bounded judgment` section — no package copies this text; each adds
only its local specialization (which J2 decision classes it delegates,
which choices are J1, which land at J0).

## Purpose

> **What authority allows this actor to decide this material question
> without returning to a human or higher authority?**

Check J5 through J0 in order. Cite the first rung that actually resolves
the decision. If two higher rungs conflict, the result is J0 — never a
silently invented precedence between them.

Governs **material** decisions: choices that affect scope, acceptance,
user-visible behavior, security, privacy, authority, destructive action,
irreversible state, promoted artifacts, or a downstream stage's
interpretation. Trivial tool mechanics do not require a citation unless
the stage contract says otherwise.

## J5 — Governing constraint

Binding law, safety policy, repository doctrine, an authority boundary, a
workflow prohibition, or the stage's own contract requires or forbids the
action. Apply it and cite the source; a lower rung cannot override it. If
two governing constraints conflict, land at J0.

## J4 — Explicit user or bound Work decision

The user, the accepted Work intent, acceptance criteria, exclusions,
repository selection, or explicit standing authorization already decides
the question and is compatible with J5. Apply the recorded decision
without asking the user to reconfirm it. Standing authorization is scoped
— never generalized beyond what was actually granted, never overriding J5.

## J3 — Settled authoritative record

An accepted upstream artifact, ADR, prior stage output, pinned
specification, authoritative system observation, or previously adjudicated
decision settles the question. Reuse it and cite the artifact. A draft,
self-authored output, stale observation, or unsupported inference does not
qualify.

## J2 — Delegated actor judgment

The active skill or stage explicitly delegates this class of decision
within named bounds. Inspect evidence, choose, and record the rationale
and rung. "Use your best judgment" without a bounded decision class named
is not a J2 grant — the package must name the delegation.

## J1 — Local, reversible, non-contractual choice

The choice is local to the current implementation, easily reversible, and
cannot change scope, authority, security, data, public behavior,
acceptance, or another actor's contract. Choose conservatively; record the
choice only when it materially affects review or maintenance. A choice is
not J1 merely because the actor believes the risk is low.

## J0 — Not delegated, conflicting, or risk-changing

No higher rung resolves the question, evidence conflicts, authority is
missing, or the choice would change scope, policy, security/privacy
posture, destructive effects, irreversible state, public behavior,
acceptance, or promotion. Do not guess.

For a workflow stage:

1. record the unresolved decision;
2. state which rungs were checked and why they did not settle it;
3. preserve the evidence already gathered;
4. state the actor's recommended answer when one can be responsibly
   offered;
5. end the turn with one direct question so the existing backend signal
   places the Work in `needs_input`.

For a Captain skill, ask the question live and wait for the user's answer
before continuing.

Canonical shape:

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

## Conflict rule

Not a numeric override table. A user request that conflicts with binding
policy does not become valid because J4 is "below" J5 — the conflict
itself is J0 unless the governing source defines an authorized exception
process.

## Authority inheritance

Narrowing only:

```text
repository / organizational doctrine
        -> Work intent and explicit user decisions
            -> workflow authority envelope
                -> stage or skill specialization
                    -> actor decision
```

A stage may narrow its workflow. A skill loaded by a stage may narrow the
stage. Neither may widen the parent contract.

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
that this was over-escalation, per §14.3's named risk.
