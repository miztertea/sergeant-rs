# 01-cleanup-preconditions: Cleanup Preconditions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Trigger: sgt-cleanup is invoked for a task. Outcome: cleanup proceeds only once every named precondition holds, and never as a shortcut for a nonterminal worker state.

Source evidence: `BU-0171` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
