# Author Document

Layer 1 orientation only — never delivered as a stage's instructions;
each stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`docs/icm/convention.md` §1a rule 5).

## Purpose

Produce a document as the deliverable, with fidelity to the brief as the
top-weighted review axis.

## Trigger

A document is the deliverable — a report, a spec, an ADR-shaped write-up,
or a transcription of decisions an in-session grilling already made.

## Stages

| Stage | Rung | Durable outcome |
|---|---|---|
| `00-map-sources` | actor-stage | The authoritative source material is inventoried by path, with what each source is authoritative for. |
| `10-establish-structure` | actor-stage | An outline against a named audience and purpose. |
| `20-draft` | actor-stage | The draft, written only from mapped sources. |
| `30-verify-fidelity-and-facts` | actor-stage | Fidelity-to-brief as the top-weighted axis, plus factual and internal consistency. |
| `35-challenge` | actor-stage | Adversarial read: what does this document fail to accomplish for its audience? |
| `40-finalize` | actor-stage | Edited for audience; closed with evidence (which sources, which revision of each). |

## Relationships to other workflows

This workflow recommends, and never dispatches, follow-up work its own
document implies (a decision the draft surfaces as needing further
investigation becomes a recommended `investigate` intent, never in-scope
work this Work absorbs). There is no child-workflow dispatch and no
worker-side submission (`docs/icm/convention.md` §7.5).

## Authority envelope

This workflow receives an already-admitted Work whose intent names a
document to produce, and — when the `record-decisions` profile applies —
a brief carrying decisions an in-session grilling already made.

### May decide
- Which sources are authoritative for which claims (`00`).
- The outline's structure against the named audience and purpose (`10`).
- Wording and structure within the draft, provided it stays sourced
  (`20`).

### May not decide
- Make a new decision, resolve an open question, or pick between
  alternatives the source brief left unresolved — where the
  `record-decisions` profile applies, this workflow transcribes and
  reviews decisions already made; it does not make them (`10`, `30`).
- Invent a rationale, fact, or source the mapped material does not carry,
  to make the document look complete (`20`, `30`).
- Treat a challenge as answered merely because it was not explicitly
  addressed (`35`) — default-refuted applies to challenges here as
  elsewhere.

### Human or Captain gates
- A source cannot be verified for a claim the draft needs (`30`) — the
  gap is stated in the document, never smoothed over.
- Where the `record-decisions` profile applies: the brief does not
  clearly carry a decision's rationale, alternatives, or rejection
  reason — logged as a gap, not escalated by default, unless the missing
  material makes the decision itself unidentifiable.

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence; the finalized document itself, with its source list and
revisions, is this workflow's central decision record.

## Robustness

**(a)** Six checkpoints; the mapped sources at `00` and the outline at
`10` are banked so a stall during drafting does not require re-mapping or
re-outlining.

**(b)** `35-challenge` attacks the draft adversarially, after
`30-verify-fidelity-and-facts` has already checked it against its own
brief — two different attacks, fidelity-to-brief and fitness-for-
audience, neither substituting for the other.

**(c)** A source that cannot be verified becomes a stated gap in the
document, never a smoothed-over assertion (`30`'s own contract); a
challenge that cannot be answered is recorded as an open challenge in
`40-finalize`'s evidence, not silently dropped.

## Notes for reviewers

**The `record-decisions` profile.** Absorbed here as a named section
inside `10-establish-structure` and `30-verify-fidelity-and-facts`, not
as a separate package and not as a separate stage. Its contract is
preserved in substance: the input is a brief carrying decisions already
made — the record of an in-session grilling, not a request to make new
decisions; a missing rationale is logged, never invented; fidelity to the
brief is the top-weighted axis.

**The honesty note this package carries.** Sergeant has no profile
construct. "Profile" here means a section inside these stage contexts.
The alternative — a second package — is a fork, which is what the rule
"profiles specialize; they do not fork" forbids, and at two stages it
would fail this wave's own robustness bar anyway (a package that cannot
answer P4's three questions is a stage or a policy in a workflow's
directory layout, not a workflow of its own). This is flagged as a
ratify-at-review item on the head PR.
