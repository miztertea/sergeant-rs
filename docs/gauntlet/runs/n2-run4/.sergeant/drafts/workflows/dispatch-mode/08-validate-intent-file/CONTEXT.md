# 08-validate-intent-file

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a dispatch objective touches any of the named sensitive categories

**Outcome:** dispatch requires a validated intent file before any mutating dispatch action, and validation failures block before mutation

**Statement (the operative rule):** `--intent-file` is required whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; the file must contain the eight required sections, and malformed, missing, traversing, symlinked, or oversized input fails before dispatch mutation.

## What must become true here (durable outcome)

Dispatch requires a validated intent file before any mutating dispatch action, and validation failures block before mutation — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0327`: An intent file path is rejected as unsafe if any path component along its resolved chain — not just the final component — is itself a symlink.
- `BU-0328`: An intent document is only valid if it contains exactly the eight required sections (Objective, Required Invariants, Approved Tradeoffs, Out Of Scope, State Transitions, Failure Windows, Negative Test Matrix, Validation Evidence), each appearing exactly once, in that exact order, and none left empty.
- `BU-0329`: An operator-supplied intent file is rejected if its path contains a newline or carriage return, or if it attempts path traversal (".." as a whole component or embedded), or if it does not exist.
- `BU-0330`: An operator-supplied intent file is rejected if it exceeds 65536 bytes, or if it contains control-character bytes other than tab and newline.
- `BU-0331`: A dispatch objective is rejected — requiring an explicit --intent-file instead — when it matches safety-sensitive or stateful keywords (auth, oauth, security, secrets, credentials, payments, databases, migrations, stateful, production, destructive, or persistent/state-transition phrasing).
- `BU-0332`: The synthesized standard-isolated intent (used when no --intent-file is supplied and the objective is not safety-sensitive) explicitly authorizes no persistent or externally published state transition, and requires stopping on any native validation, review, or dispatch failure without publishing partial work.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0333`: Installing a prepared intent file into its target path is done by copying to a temp file in the target directory and then renaming it into place, never by writing the target path directly.

