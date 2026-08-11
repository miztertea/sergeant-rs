# 02-respond-to-worker: Respond To Worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-evaluate-wake-condition/output/ | L4 | upstream artifact from this candidate's own prior stage (evaluate-wake-condition); shape to be fixed at promotion review, see ../01-evaluate-wake-condition/output/README.md |

## Purpose

Trigger (BU-0155): sgt-respond is about to be used. Trigger (BU-0275): a worker escalates with needs_input/blocked. Outcome: the five-step precondition/delivery sequence runs, and the human decision is genuinely obtained (not inferred) before a response is sent.

Source evidence: `BU-0155, BU-0275` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0157` -- a delivered response is applied exactly once, matching ID/generation/status
- `BU-0177` -- a pending response is never clobbered; the correct convergence path is used instead of recover

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
