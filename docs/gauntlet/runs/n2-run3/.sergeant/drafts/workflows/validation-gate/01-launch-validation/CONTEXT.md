# 01-launch-validation: Launch Validation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Trigger: a dispatched worker reaches readiness / sgt-validate is invoked at readiness. Outcome: a validation-only boundary runs in a coordinator-owned, split pane, never auto-approved, with redundant stages skipped by a defined default set.

Source evidence: `BU-0042, BU-0161` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0162` -- exactly one launch per task/repo pair, concurrent attempts fail closed
- `BU-0163` -- default transport never exposes intent via argv
- `BU-0164` -- missing --intent-file support fails closed with full diagnostic, no partial state
- `BU-0165` -- an argv-exposure consent applies to exactly one invocation, cannot silently persist
- `BU-0166` -- transport choice is durably auditable and the executing build is re-verified against it
- `BU-0169` -- rollback on pre-commit failure is scoped strictly to provably-owned artifacts

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
