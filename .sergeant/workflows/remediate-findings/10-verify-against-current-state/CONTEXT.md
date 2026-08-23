# 10-verify-against-current-state: verify against current state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-ingest/output/ingest.md | L4 | the accepted finding set this stage checks against current state |

## Purpose

Each finding is checked against the current revision; stale findings are
marked as such with evidence.

## What must become true here (durable outcome)

Every ingested finding has been checked against the current revision;
each one is marked either still-live or stale, with evidence either way.

## Behavior contract

- **Re-check each finding's claim against the current state of the code**
  — the finding may have been produced against a revision that has since
  moved (a colleague's PR merged something adjacent, or an earlier fix
  already addressed it).
  (trigger: this stage begins; outcome: `20-dispose` decides dispositions
  against current reality, not a stale snapshot)
- **A finding found stale is marked so with evidence** — what changed,
  and why the finding no longer applies as originally stated.
  (trigger: a finding no longer holds; outcome: `20-dispose` can dispose
  it `superseded` with a traceable reason)
- **This stage attacks the findings themselves; it does not dispose them.**
  Marking a finding stale is evidence for `20-dispose`'s decision, not
  the decision itself.
  (trigger: verification is complete; outcome: disposition stays a
  distinct, later act)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Judging whether a finding still applies against current state, and what
evidence establishes that.

### J1 — local choices allowed
Exact wording of the verification record.

### J0 — must become `needs_input`
A finding's staleness cannot be resolved from current state alone (e.g.
the underlying code moved in a way that makes the original claim
ambiguous rather than clearly stale or clearly still-live).

### Completion boundary
This stage may complete only once every ingested finding has a
still-live/stale determination with evidence.

### Decision evidence
`output/verification.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
