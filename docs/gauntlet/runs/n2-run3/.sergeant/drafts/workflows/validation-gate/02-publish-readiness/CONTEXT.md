# 02-publish-readiness: Publish Readiness

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-launch-validation/output/ | L4 | upstream artifact from this candidate's own prior stage (launch-validation); shape to be fixed at promotion review, see ../01-launch-validation/output/README.md |

## Purpose

Trigger: native validation and independent reviews all pass. Outcome: readiness is durably recorded with intent/head/review evidence before the coordinator is notified.

Source evidence: `BU-0160` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0309` -- readiness evidence anchored to a committed HEAD, never a working-tree diff

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
