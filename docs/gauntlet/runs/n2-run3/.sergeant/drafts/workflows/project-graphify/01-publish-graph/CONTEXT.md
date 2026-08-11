# 01-publish-graph: Publish Graph

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | orientation for this candidate workflow |

## Purpose

Trigger: extraction produces zero matched repos, or any repo's extraction fails. Outcome: the run stops before publication rather than silently merging and publishing an incomplete graph.

Source evidence: `BU-0250` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
