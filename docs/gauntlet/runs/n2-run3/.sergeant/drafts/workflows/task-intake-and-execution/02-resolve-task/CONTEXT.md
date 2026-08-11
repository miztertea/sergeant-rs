# 02-resolve-task: Resolve Task

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-resolve-context/output/ | L4 | upstream artifact from this candidate's own prior stage (resolve-context); shape to be fixed at promotion review, see ../01-resolve-context/output/README.md |

## Purpose

Trigger: context has been loaded. Outcome: an existing canonical td task is reused, or a new one created only otherwise.

Source evidence: `BU-0026` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
