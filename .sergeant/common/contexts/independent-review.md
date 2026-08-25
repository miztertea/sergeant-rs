# Independent review

Resolved as `@@independent-review` from
`.sergeant/common/contexts/independent-review.md` per
`.sergeant/common/contexts/icm-policy.md` §4. Policy context (Placement Ladder PL-3).
Consumers: `validate-and-ship/40-drive-gates`, `validate-and-ship/60-close-out`
(in place of the scattered restatements of the never-edit rule this
package previously carried inline), and every panel-bearing package's own
`## Authority envelope`.

## The rule

The reviewing actor is not the implementing actor. Findings default to
refuted (`@@refute`). **The validating actor never edits the code.**

## Where independence actually lives

`.sergeant/common/contexts/icm-policy.md` §6.3: a later stage in the *same* workflow may
qualify as independent review when it has a fresh execution, explicit
inputs (not inherited conversation state), a review-only contract, and no
authority to edit the subject it reviews. Independence lives in the
execution boundary, not in whether the reviewer happens to share a
workflow wrapper with the work it reviews.

**Stated honestly, not oversold.** An in-stage panel (`@@panel`,
`@@fan-out-evidence`) spawns several sub-agent seats inside a single
stage execution — a single execution boundary shared by every seat (see
`@@panel`'s own isolation-honesty paragraph for the full statement). The
property this policy names —
independence anchored in a fresh execution — is only partly available
today, and this is exactly where: a genuinely separate Work would give it
in full; a same-Work later stage gives it in part; a same-stage spawned
seat gives context isolation only. A package invoking this policy states
which of the three its own mechanism actually provides, rather than
claiming the strongest reading by default.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** the reviewing actor is asked to edit the
  code it is reviewing, or to approve its own prior work as if it were
  independent — refuse and escalate; this is not a local judgment call.
- **J2 the caller retains:** how to phrase its own `## Authority envelope`
  statement of which independence property its mechanism provides.
- **J1 the caller retains:** none beyond ordinary tool mechanics.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
