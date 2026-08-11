# 07-apply-outcome

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** an item's disposition is decided to be `ready-for-agent`

**Outcome:** an agent brief comment is posted

**Statement (the operative rule):** When an item's outcome is `ready-for-agent`, the triage skill posts an agent brief as a comment on the issue or PR.

## What must become true here (durable outcome)

An agent brief comment is posted — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1162`: When an item's outcome is `ready-for-human`, the triage skill posts a brief with the same structure as an agent brief, but additionally states why the work can't be delegated to an agent (judgment calls, external access, design decisions, manual testing).
- `BU-1163`: When an item's outcome is `needs-info`, the triage skill posts triage notes using the needs-info template.
- `BU-1164`: When an item is closed `wontfix` because the requested change already exists in the codebase, the triage skill points to where it already lives and does not write to `.out-of-scope/` — that knowledge base is only for rejected requests, not built ones.
- `BU-1165`: When a bug report is closed `wontfix` as rejected, the triage skill posts a polite explanation and then closes it.
- `BU-1166`: When an enhancement request is closed `wontfix` as rejected, the triage skill writes an entry to `.out-of-scope/`, links to it from a closing comment, and then closes the item.
- `BU-1167`: When an item's outcome is `needs-triage`, the triage skill applies that role, with an optional comment if there is partial progress to record.
- `BU-1173`: Questions posted in triage notes must be specific and actionable, not a generic ask like "please provide more info".
- `BU-1175`: An agent brief is the authoritative specification an AFK agent works from when an item moves to `ready-for-agent`; the original issue/PR body and discussion are context only, not the operative contract.
- `BU-1176`: An agent brief's scope differs by surface under the same principles: for an issue it covers building the change from nothing, and for a PR it covers what's left to do to the existing diff — finishing it, closing gaps, addressing review points.
- `BU-1177`: An agent brief must describe interfaces, types, and behavioral contracts (naming specific types, function signatures, or config shapes), and must not reference file paths, line numbers, or assume the current implementation structure will persist — because the codebase may change before the brief is picked up.
- `BU-1178`: An agent brief describes what the system should do, not how to implement it — the agent explores the codebase fresh and makes its own implementation decisions.
- `BU-1179`: Every agent brief must have concrete, testable acceptance criteria, with each criterion independently verifiable.
- `BU-1180`: An agent brief must state what is out of scope, to prevent the agent from gold-plating or making assumptions about adjacent features.
- `BU-1181`: For a PR-targeted agent brief, "current behavior" describes the state of the existing diff, and the brief asks the agent to finish or fix that diff rather than build the change from scratch.
- `BU-1183`: An `.out-of-scope/` file is written in a relaxed, readable style — more like a short design document, using paragraphs, code samples, and examples — rather than a terse database-entry format.
- `BU-1185`: The reason recorded in an `.out-of-scope/` file must be substantive — referencing project scope/philosophy, technical constraints, or a strategic decision — not a bare "we don't want this".
- `BU-1186`: The reason recorded in an `.out-of-scope/` file must be durable — a temporary-circumstance excuse ("we're too busy right now") is a deferral, not a real rejection, and should not be recorded as one.
- `BU-1191`: The triage skill writes to `.out-of-scope/` only when an enhancement (not a bug) is rejected as `wontfix`; this applies equally to a rejected enhancement PR, which is recorded so the same request doesn't return as fresh code.
- `BU-1192`: The triage skill never writes to `.out-of-scope/` when an item is closed `wontfix` because it is already implemented — that would poison the deduplication checks with a false rejection; instead the closing comment points to where the feature already lives.

