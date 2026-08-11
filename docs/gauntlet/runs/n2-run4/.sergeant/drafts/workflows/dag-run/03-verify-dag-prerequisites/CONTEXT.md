# 03-verify-dag-prerequisites

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../02-advance-on-dependency-completion/output/outcome.md | L4 | upstream evidence produced by `advance-on-dependency-completion` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the DAG runner is not on PATH when the DAG-run step is invoked

**Outcome:** the run fails closed with actionable install guidance rather than failing deep inside a later DAG-runner call

**Statement (the operative rule):** The DAG-run step refuses to run if the DAG runner binary is not installed, reporting it as an optional dependency with install instructions, and states that all other Sergeant commands work without the DAG runner.

## What must become true here (durable outcome)

The run fails closed with actionable install guidance rather than failing deep inside a later DAG-runner call — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0860`: The DAG-run step refuses to run if yq is not installed.
- `BU-0861`: The DAG-run step refuses to run if the named project's YAML config file does not exist.
- `BU-0862`: The DAG-run step refuses to run if the project's YAML config has no dag.name field.
- `BU-0863`: The DAG-run step refuses to run if the project's dag block defines zero stages.
- `BU-0864`: In --dry-run mode, the DAG-run step only prints what it would create or update — no DAG runner dag/stage/run mutation call is ever made, per-stage dry-run prints included, and the script exits before starting a run.
- `BU-0870`: After the DAG runner has already started the run, the DAG-run step best-effort parses the run ID out of the DAG runner's own output (trying a UUID pattern, then a 'started run' token pattern), and falls back to printing a literal placeholder in the monitor-command hint if neither parse succeeds — a failure to parse the ID never undoes or reports the run as failed.

