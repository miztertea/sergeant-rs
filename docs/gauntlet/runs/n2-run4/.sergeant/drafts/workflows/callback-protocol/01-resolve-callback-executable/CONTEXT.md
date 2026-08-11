# 01-resolve-callback-executable

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a callback profile is installed or invoked

**Outcome:** only a locally-installed, ownership-and-permission-verified executable can ever run as a callback, never a path supplied through request/fleet data

**Statement (the operative rule):** A callback profile executable must be real (not a symlink), owned by the Sergeant user, not group/world writable, and have its owner execute bit set; Sergeant never accepts an executable path from fleet state.

## What must become true here (durable outcome)

Only a locally-installed, ownership-and-permission-verified executable can ever run as a callback, never a path supplied through request/fleet data — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0215`: `SERGEANT_CALLBACKS` may select a different fixed callbacks directory for tests or an isolated service account, but that environment setting is itself treated as trusted local configuration, never as request input.
- `BU-0762`: Every directory the callback-delivery step trusts (fleet root, task directory, callbacks root, callback event directories) must be a real directory, not a symlink, owned by the invoking user, and — unless explicitly marked writable-ok — not group- or world-writable, or the operation fails.
- `BU-0764`: Before a callback profile can be registered against or invoked for any task, its executable file must be verified as a real, non-symlink, user-owned, non-group/world-writable file that is executable by its owner — any violation is reported as the profile not being installed or not executable, not silently accepted.
- `BU-0766`: Reading a trusted file re-checks, on the opened file descriptor, that it is still a regular file owned by the current user (and, where a mode is required, has exactly that mode) after the open — not only via the pre-open lstat — and rejects a file exceeding its declared maximum size rather than silently truncating it.

