# 03-reconcile-before-start: Reconcile Before Start

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-resolve-task/output/ | L4 | upstream artifact from this candidate's own prior stage (resolve-task); shape to be fixed at promotion review, see ../02-resolve-task/output/README.md |

## Purpose

Trigger: an execution mode has been chosen, before starting work. Outcome: existing state is reconciled and reused rather than duplicated.

Source evidence: `BU-0028` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
