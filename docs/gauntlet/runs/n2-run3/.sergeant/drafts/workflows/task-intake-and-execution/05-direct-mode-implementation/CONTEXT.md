# 05-direct-mode-implementation: Direct Mode Implementation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-execute/output/ | L4 | upstream artifact from this candidate's own prior stage (execute); shape to be fixed at promotion review, see ../04-execute/output/README.md |

## Purpose

Trigger: direct mode is active. Outcome: the owning td task is claimed/created and implementation proceeds test-first.

Source evidence: `BU-0012` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0010` -- context/td state loaded before any edit
- `BU-0011` -- in-progress work by other workers/worktrees reconciled, not duplicated/raced

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
