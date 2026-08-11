# Workflow-level helpers — dag-run

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=dag-run`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-0201`: A project's `dag.name` must be unique across all projects known to the DAG runner, since the DAG-run step runs it as a DAG identified by that name.
- `BU-0579`: The interactive fleet-watch loop only advances a linked DAG runner DAG run when both dagr_run_id and dagr_stage_id are recorded for a repo and the DAG runner binary is available on PATH; otherwise it silently does nothing.
- `BU-0580`: When advancing a linked DAG runner DAG run, the interactive fleet-watch loop reports the literal result 'done' only for a done worktree status; every other terminal status is passed through verbatim as the step result string.
- `BU-0865`: Creating the DAG runner DAG is idempotent: an error from the DAG runner because the DAG already exists is deliberately ignored rather than failing the run.
- `BU-0866`: Adding each DAG runner stage is idempotent: an error from the DAG runner because the stage already exists is deliberately ignored rather than failing the run.
