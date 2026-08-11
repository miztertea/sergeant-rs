# 03-phase2-install-checkout

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-phase1-detect-prerequisites/output/outcome.md | L4 | upstream evidence produced by `phase1-detect-prerequisites` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** no local Sergeant clone exists

**Outcome:** the destination is confirmed with the user before any clone command runs

**Statement (the operative rule):** If the Sergeant repository is not already cloned, the skill first asks the user where to place the clone and waits for an answer before proceeding to the next step.

## What must become true here (durable outcome)

The destination is confirmed with the user before any clone command runs — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1275`: The skill shows the exact `git clone` command and destination and asks for consent; the command runs only after the user types `y` or `yes`, leaving the filesystem unchanged on any other response.
- `BU-1276`: If the toolchain/task runner is available, the skill determines the actual install directory, shows the resolved target, and asks for consent before running the toolchain/task runner; if the toolchain/task runner is unavailable or consent is declined, it instructs the user to symlink commands from `bin/` manually and verify the result before continuing.
- `BU-1277`: The skill verifies that at least the fleet-listing step, the project context-resolution step, the dispatch step, and the interactive fleet-watch loop resolve on `PATH` before proceeding, reports any missing commands and their expected source path, and stops the current run if verification fails after install instructions were followed — the next run re-checks Phase 2.

