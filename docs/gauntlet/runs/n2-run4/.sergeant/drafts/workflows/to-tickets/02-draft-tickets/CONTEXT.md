# 02-draft-tickets

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-load-ticket-context/output/outcome.md | L4 | upstream evidence produced by `load-ticket-context` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a behavior spans multiple layers (storage, API, UI/CLI, tests)

**Outcome:** one ticket covers the full vertical slice rather than being split by layer, and is independently verifiable when done

**Statement (the operative rule):** Vertical slice rules require including every necessary layer for one behavior and forbid creating separate horizontal tickets (e.g. "write backend", "write frontend", "add tests") for that one behavior; a completed ticket must be demoable, testable, or operationally verifiable alone.

## What must become true here (durable outcome)

One ticket covers the full vertical slice rather than being split by layer, and is independently verifiable when done — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1312`: Decisions already approved are not reopened as questions when extracting decisions and unknowns.
- `BU-1313`: A short investigation ticket is created only when an unknown cannot be answered from existing evidence, and it must name the decision or artifact it produces.
- `BU-1314`: A ticket's `Blocked by` field lists only tickets that truly prevent starting or merging this work.
- `BU-1315`: A ticket's `Preserved state` field records the branch, commit, PR, or worktree needed to resume the work.
- `BU-1317`: Prefactoring is put first only when it materially reduces risk for the slices that follow it.
- `BU-1318`: Wide refactors follow expand (add the new form beside the old), migrate (move callers in bounded, green batches), then contract (remove the old form after every migration ticket completes).
- `BU-1319`: Migrate tickets are declared blocked by expand, and the contract ticket is declared blocked by every migration ticket.
- `BU-1333`: A ticket must not include a brittle implementation file list unless a preserved prototype requires it.

