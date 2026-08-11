# 01-verify-prerequisites

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the dependency check is being run during installation

**Outcome:** installation does not proceed until both the td-implementation check and the agent-availability check pass

**Statement (the operative rule):** Setup continues only once `td create --help` shows Marcus `td` support for `--description`, `--json`, and `--work-dir`, and at least one supported agent resolves on PATH.

## What must become true here (durable outcome)

Installation does not proceed until both the td-implementation check and the agent-availability check pass — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0132`: Sergeant requires the Marcus `td` implementation with JSON, task creation, and `--work-dir` support; a different executable named `td` is rejected.
- `BU-0173`: If another executable named `td` is first on PATH, PATH is corrected rather than wrapping unsupported output indefinitely; `td create --help` must show `--description`, `--json`, and `--work-dir`.
- `BU-0209`: The dependency check accepts `td` as present only when its version output is a supported version AND `td create --help` shows all three of `--description`, `--json`, and `--work-dir`; missing or unsupported td fails the check.
- `BU-0210`: The dependency check accepts the first of `opencode`, `goose`, or `claude` found on PATH as the agent harness and reports success; if none is found, it reports the check as failed.
- `BU-0211`: The dependency check exits nonzero and instructs the user to install missing required dependencies when any required check failed; it does not proceed with Sergeant-using tasks on that state.
- `BU-0915`: The output of 'td --version' is accepted only if it is exactly Marcus td's plain single-line version string or its exact three-line update-available notice with internally consistent version numbers; any other or mixed output is rejected before dispatch creates any side effects.
- `BU-0916`: Before any td-backed dispatch, Sergeant verifies both that the installed td's version output is recognized and that its 'td create --help' output advertises the required --description, --json, and --work-dir flags; failing either check dies naming the unsupported binary's path and version.

