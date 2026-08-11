# 08-reconcile-and-deliver: Reconcile And Deliver

## Inputs

| File | Layer | Why |
|---|---|---|
| ../07-direct-mode-delivery/output/ | L4 | upstream artifact from this candidate's own prior stage (direct-mode-delivery); shape to be fixed at promotion review, see ../07-direct-mode-delivery/output/README.md |

## Purpose

Trigger: work has reached a terminal or deliverable state. Outcome: cleanup runs only after terminal state and evidence preservation are verified.

Source evidence: `BU-0035` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
