# 30-fix-confirmed: fix confirmed

## Inputs

| File | Layer | Why |
|---|---|---|
| ../25-refute/output/findings.md | L4 | the finding set this stage fixes from — confirmed findings only |

## Purpose

Every `confirmed` finding is either fixed in a named commit or recorded as
unfixed with a reason. Nothing else changed.

## What must become true here (durable outcome)

Every finding at `status: confirmed` in `output/findings.md` (from
`25-refute`) is either linked to the commit(s) that fix it, or recorded as
`confirmed` and deliberately unfixed with a stated reason — and nothing
else in the diff has changed.

## Behavior contract

Apply `@@fix-confirmed`. This package's own narrowing:

- **Only findings whose `status` is `confirmed` may be fixed. A `refuted`
  finding is not fixed, not partially fixed, and not addressed in
  passing.**
  (trigger: this stage begins; outcome: the fixer's scope is exactly the
  set that survived attack)
- **No opportunistic change: no refactoring, no adjacent cleanup, no
  rename, no dependency bump, no "while we're in here". Anything else the
  fixer notices is recorded as a recommended follow-up intent in
  `output/fixes.md`.**
  (trigger: the fixer sees an unrelated improvement; outcome: the fix diff
  stays reviewable against the finding set, and scope does not broaden)
- **Each confirmed finding is recorded with the commit or commits that
  address it, by id. A finding the fixer cannot or should not fix is
  recorded as `confirmed` and unfixed, with the reason.**
  (trigger: closing out a finding; outcome: `35-re-verify` has an exact
  list of fix commits to attack, and no finding disappears)
- **A fix that requires changing what the change is for — its acceptance,
  its scope, its declared invariants — is not made here.**
  (trigger: the only available fix alters the intent; outcome: J0
  escalation, because scope is non-transferable)
- **The fixer re-runs the targeted validation named at `05-baseline`
  after its last commit and records the real output.**
  (trigger: fixes are complete; outcome: `35-re-verify` starts from a
  known state rather than an assumed one)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to fix a given confirmed finding, within the never-opportunistic
bound above.

### J1 — local choices allowed
Ordering of otherwise-independent fixes.

### J0 — must become `needs_input`
A confirmed blocker cannot be fixed within the intent's scope, or two
confirmed findings' fixes contradict each other.

### Completion boundary
Every `confirmed` finding is either linked to a commit or recorded as
unfixed with a reason; nothing else is in the diff.

### Decision evidence
`output/fixes.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
