# 02-create-td-tasks: Create Td Tasks

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-intent-file-gate/output/ | L4 | upstream artifact from this candidate's own prior stage (intent-file-gate); shape to be fixed at promotion review, see ../01-intent-file-gate/output/README.md |

## Purpose

All-or-nothing td task creation across selected repos, with rollback on partial failure, explicitly before any worker is spawned.

Source evidence: `BU-0284` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
