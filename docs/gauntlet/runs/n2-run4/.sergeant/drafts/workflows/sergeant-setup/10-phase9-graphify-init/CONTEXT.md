# 10-phase9-graphify-init

## Inputs

| File | Layer | Why |
|---|---|---|
| ../09-phase8-treehouse-init/output/outcome.md | L4 | upstream evidence produced by `phase8-treehouse-init` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the graph-generation tool is available and the project YAML requests output

**Outcome:** a successful Graphify run is verified by the presence of both named output files

**Statement (the operative rule):** In Phase 9, if the graph-generation tool is present on `PATH` and the project YAML has a `graphify.output` field, the skill offers to run the graph-generation step, running it only on confirmation and skipping silently on decline, and requires both `graph.json` and `GRAPH_REPORT.md` to exist at the configured output path after a successful run.

## What must become true here (durable outcome)

A successful Graphify run is verified by the presence of both named output files — per the Statement above, which is the operative rule this stage exists to enforce.

