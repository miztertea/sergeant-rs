# 01-intent-file-gate: Intent File Gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Gates any mutating dispatch action behind a validated intent file when the objective touches a sensitive category -- the earliest possible checkpoint, since it must run before task creation or worker spawn.

Source evidence: `BU-0140` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
