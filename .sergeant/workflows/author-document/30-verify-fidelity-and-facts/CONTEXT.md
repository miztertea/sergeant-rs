# 30-verify-fidelity-and-facts: verify fidelity and facts

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-map-sources/output/sources.md | L4 | the sources every fact check is verified against |
| ../10-establish-structure/output/structure.md | L4 | the audience/purpose and, where applicable, the transcribed decisions this stage checks fidelity against |
| ../20-draft/output/draft.md | L4 | the draft this stage verifies |

## Purpose

Fidelity-to-brief as the top-weighted axis, plus factual and internal
consistency. A source that cannot be verified becomes a **stated gap in
the document**, never a smoothed-over assertion.

## What must become true here (durable outcome)

Every factual claim in the draft is checked against its mapped source;
every internal-consistency issue is found; and — where the intent named a
brief the outline was built from — fidelity to that brief is checked as
the top-weighted finding, outranking every other axis.

## Behavior contract

- **Fidelity to the brief is the top-weighted axis wherever a brief
  governs**: a fidelity finding against the draft outranks a finding on
  any other axis, and a draft with an open fidelity finding is never
  considered complete regardless of how well it scores otherwise.
  (trigger: the draft is checked; outcome: a well-written document that
  quietly drifted from its brief is never mistaken for a good one)
- **Every factual claim is checked against the source `00-map-sources`
  mapped it to.** A claim that does not trace to a source, or whose
  source does not actually support it, is a finding.
  (trigger: reading the draft against its sources; outcome: fact-checking
  is source-anchored, not a vibe check)
- **A source that cannot be verified becomes a stated gap in the
  document — never a smoothed-over assertion.** The draft says "this
  could not be confirmed" rather than presenting an unverified claim as
  settled.
  (trigger: a claim's source cannot be confirmed; outcome: the document's
  own honesty about its evidence base is preserved)
- **Internal consistency** — the draft does not contradict itself across
  sections — is checked alongside fidelity and facts, though it never
  outranks a fidelity finding.
  (trigger: the draft is read end to end; outcome: cross-section
  contradictions are caught before `35-challenge`)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Judging whether a given claim's source actually supports it, and how to
phrase a fidelity or fact-check finding.

### J1 — local choices allowed
Formatting of the findings record.

### J0 — must become `needs_input`
A fidelity finding cannot be resolved by returning to
`10-establish-structure` or `20-draft` — e.g. the governing brief is
itself genuinely ambiguous about what was decided.

### Completion boundary
This stage may complete only once every claim has been checked against
its source, every unverifiable source is recorded as a stated gap, and no
open fidelity finding remains.

### Decision evidence
`output/verification.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
