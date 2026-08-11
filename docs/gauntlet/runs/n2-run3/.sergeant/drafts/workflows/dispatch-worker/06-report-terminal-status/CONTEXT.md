# 06-report-terminal-status: Report Terminal Status

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-escalate-undecided-seam/output/ | L4 | upstream artifact from this candidate's own prior stage (escalate-undecided-seam); shape to be fixed at promotion review, see ../05-escalate-undecided-seam/output/README.md |

## Purpose

Triggered when a worker reaches a terminal outcome -- the last checkpoint in the sequence.

Source evidence: `BU-0283` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
