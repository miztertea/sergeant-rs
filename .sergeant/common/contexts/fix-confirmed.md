# Fix confirmed

Resolved as `@@fix-confirmed` from
`.sergeant/common/contexts/fix-confirmed.md` per
`.sergeant/common/contexts/icm-policy.md` §4. Shared stage context, two or more consumers:
`implement-change/30-fix-confirmed`, `fix-defect/40-fix-with-regression-test`,
`remediate-findings/30-implement-accepted`.

Fix exactly what survived attack. Nothing else.

## Contract

- **Confirmed only.** A `refuted` finding is not fixed, not partially
  fixed, and not addressed in passing because the fixer happened to be in
  that part of the file anyway.
- **No opportunistic change.** No refactoring, no adjacent cleanup, no
  rename, no dependency bump, no "while we're in here". Anything else the
  fixer notices is recorded as a recommended follow-up intent, never
  folded into this diff.
- **Commit-per-finding, recorded by id.** Each confirmed finding is linked
  to the commit(s) that address it. A finding the fixer cannot or should
  not fix is recorded as `confirmed` and unfixed, with the reason —
  nothing disappears from the set.
- **Scope is non-transferable.** A fix that requires changing what the
  change is for — its acceptance, its scope, its declared invariants — is
  not made here.
- **Re-run the targeted validation** after the last fix commit and record
  the real output, so whatever attacks the fixes next starts from a known
  state.
- **Commit the fix.** `git add` and `git commit` each fix on the binding's
  own `work_branch` (the `BindingSummary` handed to this stage already
  names it). A fix that is correct but never committed is not fixed —
  nothing downstream (`@@re-verify`, `@@close`, `@@evidence-requirements`)
  can find or verify work that was never landed. Do not stop at a
  described or staged-but-uncommitted change.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a confirmed blocker cannot be fixed within
  the intent's scope, or two confirmed findings' fixes contradict each
  other — escalate rather than picking one silently.
- **J2 the caller retains:** how to fix a given confirmed finding, within
  the never-opportunistic bound above.
- **J1 the caller retains:** ordering of otherwise-independent fixes.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
