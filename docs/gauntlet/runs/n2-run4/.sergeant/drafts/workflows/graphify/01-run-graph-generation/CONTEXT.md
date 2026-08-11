# 01-run-graph-generation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the graph-generation step is run for a project

**Outcome:** the run is only considered successful once both named output artifacts exist

**Statement (the operative rule):** A project graph run requires both `graph.json` and `GRAPH_REPORT.md` to exist at the configured project output.

## What must become true here (durable outcome)

The run is only considered successful once both named output artifacts exist — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0184`: Graphify output for a project is kept to one output path outside source repositories, and an existing graph is never regenerated or moved without confirming the desired global-per-project path.
- `BU-0198`: If `graphify.output` is a directory symlink, the graph-generation step preserves the symlink and publishes into its target; Sergeant only replaces the published graph after a complete run and preserves existing `wiki/` and `memory/` directories.
- `BU-0199`: If `graphify.output` lives inside a source repo, Sergeant stages extraction outside that repo and excludes the configured output path, so published graph artifacts are never re-ingested as source.
- `BU-0245`: Publication of the merged project graph is atomic: concurrent readers observe either the complete old output or the complete new output throughout the run, never a partial or missing state, and the publish directory is placed so the final rename never crosses filesystem boundaries.
- `BU-0248`: Whenever exclusion patterns apply to a repo (because the output path or an excluded pattern falls inside it), the graph-generation step stages a filtered copy outside the repo before running the graph-generation tool extraction, so the excluded/output paths are never re-ingested as source.
- `BU-0250`: If zero repositories matched `include_groups`, or any included repository's extraction failed, graph-generation step exits with an error listing the failed repositories rather than publishing a graph built from a partial repo set.
- `BU-0251`: Before publication, the graph-generation step verifies that `graph.json`, `manifest.json`, and `GRAPH_REPORT.md` all exist and are nonempty in the staged output; if any is missing or empty, the run errors out and does not publish.
- `BU-0252`: Publication preserves existing `wiki/` and `memory/` directories under the graph-generation tool output by copying them into the newly staged output before the atomic swap, so the graph-generation tool run never destroys user-facing extensions it did not regenerate.
- `BU-0254`: When the configured output is a symlink outside all source repos, publication atomically replaces the symlink itself via a single rename (`mv -T`) pointing at a newly staged backing directory, rather than writing through the symlink into its old target in place.
- `BU-0262`: If no `graphify.output` is configured for a project, the procedure stops and requests or adds the project-level path before running Graphify, rather than inventing a default output location.
- `BU-0263`: After a Graphify run, `<graphify.output>/graph.json` and `GRAPH_REPORT.md` are required to exist before the run is treated as successful.
- `BU-0264`: Generated graph output is never published inside an owning source repository.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0196`: A repo's `name` must match `[A-Za-z0-9._-]+`, cannot contain spaces, and cannot be `.` or `..`, so Sergeant can safely prefix merged source paths with it for the graph-generation step.
- `BU-0246`: A repo name that does not match `[A-Za-z0-9._-]+`, or is `.`/`..`, is rejected for the graph-generation tool with a named error rather than being used to build a merged output path.
- `BU-0247`: graphify.output is never allowed to be identical to a source repository path; extraction refuses with a named error rather than extracting a repo into itself.
- `BU-0249`: When no supported LLM API key is set in the environment, extraction passes `--code-only` so the graph-generation tool indexes code via local AST without attempting semantic extraction, avoiding an abort when it encounters doc/paper/image files with no key configured.

