# 02-reconcile-hand-edit: Reconcile Hand Edit

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-route-finding/output/ | L4 | upstream artifact from this candidate's own prior stage (route-finding); shape to be fixed at promotion review, see ../01-route-finding/output/README.md |

## Purpose

Trigger: a stored finding card has been modified outside the router since it last wrote it. Outcome: the human-edited content is preserved (not overwritten) and flagged for human reconciliation.

Source evidence: `BU-0101` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
