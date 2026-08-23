# Remediate Findings

Layer 1 orientation only — never delivered as a stage's instructions;
each stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`docs/icm/convention.md` §1a rule 5).

## Purpose

Consume an approved typed finding set and account for every finding —
accepted, rejected, superseded, or unverifiable — with the fixes
themselves re-attacked.

## Trigger

A typed finding set exists (most often from `review-change`) and a human
has authorized acting on it.

## Stages

| Stage | Rung | Durable outcome |
|---|---|---|
| `00-ingest` | actor-stage | The finding set is loaded and refused if it is untyped, if ids are not unique, or if the intent does not name a human authorization for acting on it. |
| `10-verify-against-current-state` | actor-stage | Each finding is checked against the current revision; stale findings are marked as such with evidence. |
| `20-dispose` | actor-stage | Each finding is `accepted`/`rejected`/`superseded`/`unverifiable`, each with a reason. |
| `30-implement-accepted` | actor-stage | Fixes for `accepted` findings only, each linked to its commit(s). |
| `35-re-verify` | actor-stage | The fix commits are re-attacked and their tests audited. |
| `40-disposition-matrix` | actor-stage | Every ingested id appears exactly once with its disposition, reason, and (where accepted) its commit. |

## Relationships to other workflows

This workflow consumes a finding set most often produced by
`review-change` (never dispatched from it — `review-change` recommends,
this workflow is what the human separately dispatches after authorizing
action). This workflow itself recommends, and never dispatches, any
further follow-up its own disposition matrix implies.

## Authority envelope

This workflow receives an already-admitted Work whose intent names a
finding set and the human's authorization to act on it.

### May decide
- How to verify a finding against current state, and whether it is stale
  (`10`).
- How to fix an `accepted` finding, within `@@fix-confirmed`'s bounds
  (`30`).

### May not decide
- Act on a finding set the intent does not name a human authorization
  for — `00-ingest` refuses outright rather than proceeding on an
  assumed authorization.
- Dispose a finding on its own — every disposition requires a stated
  reason, and an unverifiable finding is disposed `unverifiable` with
  evidence, never silently accepted or dropped.
- Fix a `rejected`, `superseded`, or `unverifiable` finding.
- Self-authorize from a review's output — the human names the
  authorization; this workflow does not infer one from the finding set's
  mere existence (external rule 11; AGENTS.md's PACE rule that scope
  changes never transfer).

### Human or Captain gates
- The finding set is untyped, has non-unique ids, or the intent names no
  authorization (`00`).
- A finding's staleness can't be resolved from current state alone (`10`).
- A disposition turns on a policy or scope question (`20`).
- A new blocker at `35`.

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence; the disposition matrix (`40-disposition-matrix`) is this
workflow's central decision record and completeness proof.

## Robustness

**(a)** Six checkpoints; dispositions are banked at `20-dispose` — a
stall in `30-implement-accepted` does not lose the disposition decisions
already made.

**(b)** `10-verify-against-current-state` attacks the findings themselves
(are they still true?); `35-re-verify` attacks the fixes.

**(c)** A finding that cannot be verified at `10` is disposed
`unverifiable` with evidence — never silently accepted, never dropped;
`40-disposition-matrix` is the proof that every ingested id is accounted
for.

## Notes for reviewers

**Authorization is the load-bearing gate.** `00-ingest` refuses a finding
set the intent does not name a human authorization for — this is what
keeps a review's output from becoming self-authorizing remediation.
