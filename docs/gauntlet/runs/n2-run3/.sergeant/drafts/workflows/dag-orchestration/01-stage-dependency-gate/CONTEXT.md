# 01-stage-dependency-gate: Stage Dependency Gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Trigger: a DAG stage declares an after: dependency. Outcome: the stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by sgt-watch.

Source evidence: `BU-0203` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
