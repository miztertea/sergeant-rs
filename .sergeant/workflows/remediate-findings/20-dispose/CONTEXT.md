# 20-dispose: dispose

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-verify-against-current-state/output/verification.md | L4 | the staleness determinations this stage's dispositions are built from |

## Purpose

Each finding is `accepted`/`rejected`/`superseded`/`unverifiable`, each
with a reason.

## What must become true here (durable outcome)

Every ingested finding has exactly one disposition —
`accepted`, `rejected`, `superseded`, or `unverifiable` — each with a
stated reason, and this decision survives a later stall unchanged.

## Behavior contract

- **Every finding gets exactly one of the four dispositions, with a
  reason.** `accepted` (should be fixed), `rejected` (should not be
  fixed, with why), `superseded` (already addressed or moot, per
  `10`'s staleness evidence), or `unverifiable` (cannot be confirmed
  either way, with evidence of what was checked).
  (trigger: a finding's staleness determination is in hand; outcome: a
  clear, reasoned disposition exists for every finding, none left
  ambiguous)
- **An unverifiable finding is disposed `unverifiable` with evidence —
  never silently accepted, never dropped.**
  (trigger: a finding cannot be confirmed either way; outcome:
  `40-disposition-matrix`'s completeness proof holds even for the hardest
  case)
- **Dispositions are banked here and survive a stall in
  `30-implement-accepted`.** A crash after this stage does not lose the
  disposition decisions already made.
  (trigger: this stage completes; outcome: `30` resumes from a fixed,
  already-decided disposition list rather than re-deciding)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Assigning each finding's disposition and reason, within the four-value
vocabulary above.

### J1 — local choices allowed
Exact wording of each reason.

### J0 — must become `needs_input`
A disposition turns on a policy or scope question the workflow's own
authorization does not settle (e.g. whether a finding's fix would exceed
what the human actually authorized).

### Completion boundary
This stage may complete only once every ingested finding has exactly one
disposition and a reason.

### Decision evidence
`output/dispositions.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
