# Close

Resolved as `@@close` from `.sergeant/common/contexts/close.md` per
`.sergeant/common/contexts/icm-policy.md` §4. Shared stage context, two or more consumers:
`implement-change/40-close`, `investigate/40-close`,
`review-change/40-report`, `remediate-findings/40-disposition-matrix`,
`author-document/40-finalize`, `fix-defect/60-re-verify-and-postmortem`.

The evidence packet's shape — what a closing stage owes a reader, in
every package that has one.

## Contract

- **Which tests ran, against which revision, satisfying which acceptance
  criterion.** Not "tests passed" — the specific tests, the specific
  revision, the specific criterion each result is evidence for.
- **The finding set's final state, every id accounted for.** Nothing
  raised is allowed to vanish silently between a panel and a close packet.
- **The panel's coverage, honestly**: four axes or fewer, and if fewer,
  which axis was missing and why.
- **Every declared `promote` artifact named and confirmed present**
  (`.sergeant/common/contexts/icm-policy.md` §1a). This is the disposition act itself —
  the closing actor lists every `promote` path by name, confirms each
  exists, and states which `evidence`-class artifacts it is deliberately
  leaving on the Work branch. No deterministic finalize helper does this;
  `sgt init` does not ship one (see the finalize ruling, §1.7 of the
  design record), so the closing actor's own contract is the only place
  this policy is applied.
- **Any recommended follow-up intents**, stated as recommendations Captain
  may act on — never as work this Work has already broadened its own
  scope to include.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a declared `promote` artifact is missing
  or a finding's final disposition cannot be determined — the close
  packet says so rather than asserting completion it cannot back.
- **J2 the caller retains:** how to phrase the packet's narrative sections
  (what ran, what was found, what follows) within the shape above.
- **J1 the caller retains:** formatting and ordering of the packet.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
