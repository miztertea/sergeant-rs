# 06-handle-decision-gate: Handle Decision Gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-direct-mode-implementation/output/ | L4 | upstream artifact from this candidate's own prior stage (direct-mode-implementation); shape to be fixed at promotion review, see ../05-direct-mode-implementation/output/README.md |

## Purpose

Trigger: a worker reaches needs_input, blocked, or an ask-user gate. Outcome: only genuinely missing decisions are solicited and recorded in td; remediation continues without redundant re-asks.

Source evidence: `BU-0034` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
