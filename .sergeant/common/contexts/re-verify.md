# Re-verify

Resolved as `@@re-verify` from `.sergeant/common/contexts/re-verify.md`
per `.sergeant/common/contexts/icm-policy.md` §4. Shared stage context, two or more
consumers: `implement-change/35-re-verify`,
`fix-defect/60-re-verify-and-postmortem`, `remediate-findings/35-re-verify`.

The twice-earned rule: a fixer's own commits are the next thing to get
attacked, not just the feature they were fixing.

## Contract

- **The subject is the fix commits, named by whatever upstream stage
  recorded them** — not the whole change, not the original feature diff.
  Without that named list this stage has nothing to attack and cannot
  complete.
- **Two passes run over those commits**: a re-attack for defects the
  fixes themselves introduced, and a test-honesty audit of every test the
  fixer added or changed.
- **The test-honesty audit checks, for each new or changed test, that it
  fails against the pre-fix code** — or records explicitly that this was
  not demonstrated and why. "Tests passed" is never accepted as evidence
  on its own.
- **New findings continue the same typed set's id series**, severity-
  ranked, so one finding set spans the whole run.
- **A new blocker is a `needs_input` escalation, not a second fix round.**
  There is no loop primitive in this engine and this content does not fake
  one.
- **A clean re-verify is a positive result**, recorded as what was
  attacked, how, and what was found not to be wrong — never an empty file
  that leaves a reader unable to tell whether the stage ran at all.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a new blocker survives into the fix
  commits — escalate with the finding, its evidence, and the decision
  required; do not start a second fix round to absorb it.
- **J2 the caller retains:** how to design the re-attack and the
  test-honesty audit for the specific commits in front of it.
- **J1 the caller retains:** none beyond ordinary tool mechanics — both
  passes are required, in full, over every listed commit.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
