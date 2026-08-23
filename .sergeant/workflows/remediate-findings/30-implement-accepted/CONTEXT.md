# 30-implement-accepted: implement accepted

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-dispose/output/dispositions.md | L4 | the disposition set this stage fixes from — accepted findings only |

## Purpose

Fixes for `accepted` findings only, each linked to its commit(s).

## What must become true here (durable outcome)

Every `accepted` finding is either fixed in a named commit or recorded as
unfixed with a reason. Nothing else changed.

## Behavior contract

Apply `@@fix-confirmed`, read here as "accepted" in place of "confirmed."
A merge or rebase conflict met while fixing is resolved via
`@@resolve-conflicts`. This package's own narrowing:

- **Only `accepted` findings may be fixed.** `rejected`, `superseded`, and
  `unverifiable` findings are not fixed, not partially fixed, and not
  addressed in passing.
  (trigger: this stage begins; outcome: the fixer's scope is exactly the
  accepted set)
- **No opportunistic change.** Anything else noticed is recorded as a
  recommended follow-up intent, never folded into this diff.
  (trigger: the fixer notices an unrelated improvement; outcome: the fix
  diff stays reviewable against the accepted set)
- **Each accepted finding is recorded with the commit(s) that address
  it, by id.** A finding that cannot or should not be fixed after all is
  recorded as accepted-and-unfixed, with the reason.
  (trigger: closing out a finding; outcome: `35-re-verify` has an exact
  list of fix commits to attack)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to fix a given accepted finding, within the never-opportunistic
bound above.

### J1 — local choices allowed
Ordering of otherwise-independent fixes.

### J0 — must become `needs_input`
An accepted finding cannot be fixed within the authorized scope, or a
merge conflict `@@resolve-conflicts` cannot resolve from traced intent
alone.

### Completion boundary
Every `accepted` finding is either linked to a commit or recorded as
unfixed with a reason; nothing else is in the diff.

### Decision evidence
`output/fixes.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
