# 01-load-context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a task is brought to the session

**Outcome:** context is fully loaded before an execution mode is chosen

**Statement (the operative rule):** Step 1 of the standard workflow: run the project context-resolution step and identify the owning repository/repositories, inherited instructions, configured paths, and cross-repository dependencies before selecting an execution mode.

## What must become true here (durable outcome)

Context is fully loaded before an execution mode is chosen — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0134`: The coordinator is started from the Sergeant checkout inside tmux so `AGENTS.md` is loaded and dispatch can bind the exact coordinator identity.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0237`: For each repository, the project context-resolution step reports one of three clone-status states: cloned with its current branch, directory exists but is not a git repo, or NOT CLONED.
- `BU-0238`: When a project configures the graph-generation tool output, the project context-resolution step reports whether a built graph (`GRAPH_REPORT.md`) is available to read, or names the exact command to build one if not.
- `BU-0888`: The default development root directory and default identity are read from the user's config.yaml if the file exists and yq is installed; otherwise Sergeant falls back to its built-in defaults.

