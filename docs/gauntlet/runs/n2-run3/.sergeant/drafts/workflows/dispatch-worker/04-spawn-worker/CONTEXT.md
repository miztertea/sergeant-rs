# 04-spawn-worker: Spawn Worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-record-canonical-intent/output/ | L4 | upstream artifact from this candidate's own prior stage (record-canonical-intent); shape to be fixed at promotion review, see ../03-record-canonical-intent/output/README.md |

## Purpose

Trigger: work has been decomposed by repository. Outcome: one dispatched worker launched per repo (BU-0007); the four converging spawn-failure paths are handled without partial state (BU-0295).

Source evidence: `BU-0007, BU-0295` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0071` -- launch evidence never overclaims model readiness
- `BU-0072` -- launch evidence never overclaims variant verification

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
