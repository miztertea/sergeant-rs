# 03-gather-context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** an issue or PR is being triaged

**Outcome:** a redundancy check against the existing codebase is performed and its search scope is reported

**Statement (the operative rule):** While gathering context on an issue or PR, the triage skill searches the codebase for an existing implementation of the requested behavior by domain concept (not just the request's wording), and records where it looked.

## What must become true here (durable outcome)

A redundancy check against the existing codebase is performed and its search scope is reported — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1155`: If the redundancy check finds that the requested behavior is already implemented, the disposition becomes `wontfix` (already-implemented) rather than proceeding through the rest of ordinary triage.
- `BU-1156`: While gathering context, the triage skill also checks for prior rejection: it reads the `.out-of-scope/` knowledge base and surfaces any entry that resembles the current request.
- `BU-1174`: When resuming triage on an issue or PR that already has prior triage notes, the triage skill reads them, checks whether the reporter has answered any outstanding questions, and presents an updated picture before continuing rather than re-asking questions that are already resolved.
- `BU-1187`: During triage's gather-context step, the triage skill reads all files in `.out-of-scope/` when evaluating a new issue.
- `BU-1188`: A new issue is matched against `.out-of-scope/` entries by concept similarity, not by keyword matching.
- `BU-1189`: When a new issue matches an existing `.out-of-scope/` entry, the triage skill surfaces the prior rejection and its reason to the maintainer and asks whether they still feel the same way, rather than silently re-applying the old rejection.
- `BU-1190`: The maintainer's response to a surfaced `.out-of-scope/` match branches three ways: confirm (the new issue is appended to the file's prior-requests list and closed), reconsider (the file is deleted or updated and the issue proceeds through normal triage), or disagree (treated as related but distinct, proceeding through normal triage).
- `BU-1194`: If the maintainer reconsiders a previously rejected concept, the triage skill deletes the corresponding `.out-of-scope/` file.
- `BU-1195`: The triage skill does not reopen old issues that were closed under a since-reconsidered rejection — they remain historical records; only the new issue that triggered the reconsideration proceeds through normal triage.

