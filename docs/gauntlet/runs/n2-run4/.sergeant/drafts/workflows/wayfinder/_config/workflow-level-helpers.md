# Workflow-level helpers — wayfinder

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=wayfinder`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-1004`: If no issue tracker has been provided to the effort, wayfinder defaults to the local-markdown tracker rather than failing or guessing at a different one.
