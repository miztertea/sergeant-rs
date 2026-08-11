# Workflow-level helpers — validation-pipeline-gate

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=validation-pipeline-gate`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-0396`: The validation worker records its validation pipeline exit code verbatim as exited:<status> in the durable validation_status file and propagates the same exit code as its own process exit status.
- `BU-1259`: Exit codes for the pipeline-automation tool commands are `0` for success, no-op, or normal decision gates, `1` for `failed` or `cancelled` final outcomes, and `2` for bad usage.
