# Direct Implementation — retired (ICM-R2 HARVEST)

Draft retirement stub, mirrors the live package's own
`.sergeant/workflows/direct-implementation/CONTEXT.md` path. Not a live
edit — see `../../adjudication-draft.md` for the full package-adjudication
record this stub implements, and `docs/adr/0013-icm-r0-owner-rulings.md`
decision 6 (promotable-only review) for why this lives under `draft/`
rather than the live tree.

## Disposition

HARVEST. This package's behavior units are already owned elsewhere, or
fold into an already-admitted surface with a small textual addition:

- Routing/definition invariants (BU-P1-007, BU-P1-016, BU-P1-107) —
  `AGENTS.md` "When NOT to use `sgt`" already states the trigger; see
  `../AGENTS-md-fold.md` for the one proposed addition (the "same gates as
  dispatch, never a lighter path" restatement, currently scoped only to
  sergeant-rs's own code).
- Pre-work reconciliation (BU-P1-009, BU-P8-056) — folds into `AGENTS.md`
  Standard workflow loop step 2; see `../AGENTS-md-fold.md`.
- Context loading, claim-and-implement, validate, shipping-gate, PR/merge,
  record-outcomes (BU-P1-008, BU-P1-010 through BU-P1-014, BU-P8-058) —
  all already owned by `.sergeant/workflows/validate-and-ship/`'s
  directly-invoked entry (`00-check-scope`, `10-do-the-work`,
  `40-drive-gates`, `50-reconcile-custody`, `60-close-out`). No fold
  needed there beyond what already exists — see the adjudication record's
  Behavior-unit dispositions table for the citation-by-citation mapping.
- The eight-step ordered-procedure framing itself (BU-P8-055) — RETIRE,
  superseded by `validate-and-ship`'s own stage sequence.

On promotion, delete `.sergeant/workflows/direct-implementation/` in full
(all five stage directories, `_config/`, `index.md`, `workflow.toml`,
this `CONTEXT.md`'s live counterpart) and remove its entry from
`.sergeant/index.md`, following the same pattern already used to retire
`sergeant-help` and `grilling` out of `.sergeant/workflows/` once their
content was ported elsewhere.

## Why this package does not survive as its own artifact

Its own defining trigger — "the user explicitly asks to work in this
session, and one repository owns the complete outcome" — is exactly the
condition current `AGENTS.md` names as when `sgt run` (dispatch) is *not*
used. But the only way this package's five stages ever execute is through
`sgt run` admitting them as a durable Work. A workflow (PL-4,
`reference/proposal-icm-r-procedure-authority.md` §5.6) must produce "a
result that is meaningful independent of the original conversation
continuing," and "conversation cannot be its primary product" — this
package's entire reason to exist is that the conversation *does* continue
and owns the outcome. It cannot coherently be represented at PL-4/PL-5
while its own trigger names the condition under which dispatch is not
used. See the adjudication record's "Driver and admission boundary"
section for the full argument, including that this dispatched pilot Work
is itself an instance of the same inversion, one level up.
