# 35-re-verify: re-verify

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-implement-accepted/output/fixes.md | L4 | the fix commits this stage attacks — its only subject |

## Purpose

The fix commits are re-attacked and their tests audited.

## What must become true here (durable outcome)

Both the re-attack pass and the test-honesty audit have run over every
fix commit listed in `30-implement-accepted/output/fixes.md`.

## Behavior contract

Apply `@@re-verify`. This package's own narrowing:

- **The subject is the fix commits from `30-implement-accepted` — not the
  original review's diff and not the whole finding set.**
  (trigger: this stage begins; outcome: the re-attack lands on the code
  most likely to carry a fresh defect)
- **Two passes run: a re-attack for defects the fixes introduced, and a
  test-honesty audit of every test the fixer added or changed.**
  (trigger: fix commits are identified; outcome: both measured failure
  classes are looked for by name)
- **A new blocker is a `needs_input` escalation, not a second fix round.**
  (trigger: the re-attack finds a blocker; outcome: the human decides
  whether to extend this Work, rather than the workflow improvising
  another round)
- **A clean re-verify is recorded as a positive result**, never an empty
  file.

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to design the re-attack and test-honesty audit for the specific fix
commits in front of it.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
A new blocker survives into the fix commits.

### Completion boundary
Both passes have run over every listed fix commit.

### Decision evidence
`output/re-verify.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
