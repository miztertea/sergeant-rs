# 02-advance-on-dependency-completion

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../01-resolve-stage-brief/output/outcome.md | L4 | upstream evidence produced by `resolve-stage-brief` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a DAG stage declares an `after:` dependency

**Outcome:** the stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by the interactive fleet-watch loop

**Statement (the operative rule):** A DAG stage's `after:` list names the stages that must complete before it runs, and the interactive fleet-watch loop auto-advances the DAG when fleet tasks complete.

## What must become true here (durable outcome)

The stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by the interactive fleet-watch loop — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0869`: A dag.stages entry's `after` list is translated into the DAG runner's own --depends-on flag, so stage ordering declared in the project YAML is enforced by the DAG runner itself, not re-implemented by the DAG-run step.

