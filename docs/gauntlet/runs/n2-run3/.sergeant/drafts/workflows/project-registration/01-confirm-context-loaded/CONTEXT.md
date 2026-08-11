# 01-confirm-context-loaded: Confirm Context Loaded

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Trigger: project context loading is claimed complete. Outcome: completeness is defined by an observable evidence artifact, not merely having run the command.

Source evidence: `BU-0258` -- see `../provenance.md`.

## Stage-context (folded in from synthesis)

- `BU-0256` -- a raw-YAML read is only a fallback for a field sgt-context output doesn't surface
- `BU-0266` -- a discrepancy between sgt-context output and the raw YAML blocks progress and preserves evidence rather than silently picking a source

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
