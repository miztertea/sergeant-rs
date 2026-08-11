# 07-direct-mode-delivery: Direct Mode Delivery

## Inputs

| File | Layer | Why |
|---|---|---|
| ../06-handle-decision-gate/output/ | L4 | upstream artifact from this candidate's own prior stage (handle-decision-gate); shape to be fixed at promotion review, see ../06-handle-decision-gate/output/README.md |

## Purpose

Trigger: a direct-mode implementation is ready for delivery. Outcome: delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied.

Source evidence: `BU-0015` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0014` -- direct-mode work passes the same validation/review/gate steps as dispatched work
- `BU-0016` -- handoff/PR/merge/deployment/cleanup outcomes durably recorded

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
