# Workflow-level helpers — domain-modeling

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=domain-modeling`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-1057`: If no CONTEXT.md exists yet, create one lazily, at the point the first domain term is resolved — not in advance.
- `BU-1058`: If no docs/adr/ directory exists yet, create it lazily, at the point the first ADR is needed — not in advance.
- `BU-1076`: Which context-file structure applies is inferred rather than assumed: if CONTEXT-MAP.md exists it is read to find the contexts; if only a root CONTEXT.md exists the repo is treated as single-context; if neither exists, a root CONTEXT.md is created lazily when the first term is resolved.
