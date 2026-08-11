# 02-phase1-detect-prerequisites

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-run-checklist/output/outcome.md | L4 | upstream evidence produced by `run-checklist` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the skill checks the required prerequisite list

**Outcome:** `td` is accepted only if it is the specific Marcus implementation with the named flag support, and at least one of the three named interactive agents must be present

**Statement (the operative rule):** Phase 1 classifies each prerequisite as `present`, `installable`, or `unsupported`; the required set includes `td` specifically as the Marcus implementation, verified with `td version` and `td create --help` and required to support `--description`, `--json`, and `--work-dir`, plus at least one interactive agent among `opencode`, `goose`, or `claude`.

## What must become true here (durable outcome)

`td` is accepted only if it is the specific Marcus implementation with the named flag support, and at least one of the three named interactive agents must be present — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1270`: Optional prerequisites (the toolchain/task runner, the treehouse session manager, the graph-generation tool, the validation pipeline, `node`/`npm`) are skipped, not failed, if absent.
- `BU-1271`: For each unsupported prerequisite, the skill shows a draft task tracker issue (title, description, acceptance criteria) and asks for explicit approval; the issue is created only after the user types `y` or `yes`, and if declined the gap is reported in the summary without creating tracking work.
- `BU-1272`: The skill does not continue past Phase 1 until all required prerequisites are either present or the user has explicitly accepted the risk of proceeding without them.
- `BU-1273`: For each installable prerequisite, the skill shows the installation command and asks for explicit consent; the command runs only after the user types `y` or `yes`, and is not run on any other response.
- `BU-1296`: When a prerequisite install is declined, the skill reports what was skipped and asks whether to continue, rather than silently treating the decline as either a hard stop or an implicit continue.

