# 02-load-repo-context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-resolve-project-name/output/outcome.md | L4 | upstream evidence produced by `resolve-project-name` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a required repository is not yet cloned

**Outcome:** sync is deferred until actually needed, and a sync failure halts rather than proceeding with a missing repo

**Statement (the operative rule):** A missing required repository is synced only after the requested work is confirmed to require that repository, and the procedure stops if cloning or pulling fails.

## What must become true here (durable outcome)

Sync is deferred until actually needed, and a sync failure halts rather than proceeding with a missing repo — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0256`: A raw project YAML is read directly only when a required field is absent from the project context-resolution step output, not as a routine alternative to it.
- `BU-0258`: Completion evidence for loading project context is the project context-resolution step block showing every owning repository as cloned, plus the instructions and paths that will govern execution.

