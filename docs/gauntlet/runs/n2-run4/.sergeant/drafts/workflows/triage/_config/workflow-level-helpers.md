# Workflow-level helpers — triage

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=triage`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-1144`: A bare reference such as `#42` in a triage request is resolved to a specific issue or pull request according to the tracker config, not assumed to be one or the other.
- `BU-1182`: The `.out-of-scope/` knowledge base stores one file per rejected concept, not per issue — multiple issues requesting the same thing are grouped under one file.
- `BU-1184`: An `.out-of-scope/` file is named with a short, descriptive kebab-case concept name, recognizable enough that someone browsing the directory understands what was rejected without opening the file.
- `BU-1193`: When recording a rejection in `.out-of-scope/`, the triage skill appends the new issue to an existing matching file's prior-requests list if one already exists; otherwise it creates a new file with the concept name, decision, reason, and first prior request.
