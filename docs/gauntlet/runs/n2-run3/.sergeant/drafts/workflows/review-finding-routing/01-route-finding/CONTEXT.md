# 01-route-finding: Route Finding

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Trigger: a dispatched worker produces a review finding artifact. Outcome: actionable findings become owning-repo td tasks with durably published blocking guidance.

Source evidence: `BU-0096` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0103` -- a failed route retains parsed/sanitized findings with an exact retry command
- `BU-0312` -- a malformed/failed-routing review artifact escalates rather than being silently logged

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
