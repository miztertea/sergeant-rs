# Evidence requirements

Resolved as `@@evidence-requirements` from
`.sergeant/common/contexts/evidence-requirements.md` per
`docs/icm/convention.md` §4. Policy context (Placement Ladder PL-3).
Consumers: `implement-change/40-close`, `investigate/40-close`,
`review-change/40-report`, and every other closing stage that cites
`@@close`.

## The rule

Completion names *which* tests, against *which* revision, satisfying
*which* acceptance criterion. "Tests passed" is not completion.

- **Which tests.** Name the tests, not a count. A reader must be able to
  re-run the exact command.
- **Which revision.** The pinned point (`@@pin-fixed-point`) the result is
  claimed against — not "the current state," which drifts the moment
  someone reads the claim later.
- **Which acceptance criterion.** Every claim ties to a specific line of
  the located spec/acceptance source (`@@identify-spec-source`), or states
  plainly that no such source existed and why the claim is trusted anyway.
- **A new or changed test must be shown to fail against the pre-change
  code**, or the omission is recorded with its reason — the same
  discipline `@@re-verify`'s test-honesty pass applies to fix commits.
- **Every claim cites something a reader can re-run.** A command, a file
  span, a diff — never a bare assertion that something was checked.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a completion claim cannot be tied to a
  specific test, revision, and acceptance criterion — the stage records
  the gap rather than asserting completion around it.
- **J2 the caller retains:** how to phrase the evidence packet's narrative
  around these requirements.
- **J1 the caller retains:** formatting.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
