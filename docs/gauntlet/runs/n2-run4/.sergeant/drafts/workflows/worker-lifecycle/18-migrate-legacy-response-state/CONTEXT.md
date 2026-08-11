# 18-migrate-legacy-response-state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the recorded worktree is not yet recognized as an owned checkout and migration is attempted

**Outcome:** migration only proceeds from a worktree whose identity is provably genuine

**Statement (the operative rule):** Legacy response-state migration requires the recorded worktree path to be an absolute, existing, symlink-free directory whose canonical path is itself, containing an unsymlinked .git file; any deviation aborts migration without changing state.

## What must become true here (durable outcome)

Migration only proceeds from a worktree whose identity is provably genuine — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0407`: Legacy response-state migration requires the branch recorded in fleet state to exactly equal the worktree's actual checked-out branch; a mismatch aborts migration.
- `BU-0408`: Legacy response-state migration requires the worktree's own brief file to literally contain matching Task ID, Project, Repo, Branch, Worktree, and Fleet-state lines; any missing or differing line aborts migration.
- `BU-0409`: Legacy response-state migration requires the project configuration's repo entry to resolve to a path sharing the exact same git common directory as the worktree; a mismatch aborts migration.
- `BU-0410`: Before mutating any state, legacy migration records durable evidence of the exact identity facts it verified, and if evidence was already recorded for this repo, migration proceeds only when the newly computed evidence is byte-identical to it, refusing otherwise.
- `BU-0411`: Legacy response-state migration only fills in tracked pointer and intent files that are currently absent; it never overwrites a pointer or intent file that already exists.

