# 02-register-origin

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a correlation ID is supplied at origin registration

**Outcome:** the ID is validated to be opaque and rejects anything shaped like a real platform identifier

**Statement (the operative rule):** A correlation ID must match `^[a-z][a-z0-9._-]{7,127}$` and must not contain a 17-20 digit platform ID; a new opaque request ID is used, never a Discord guild/channel/user/message ID.

## What must become true here (durable outcome)

The ID is validated to be opaque and rejects anything shaped like a real platform identifier — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0217`: Origin registration writes only `.callbacks/origin.json` with correlation_id, profile, and version; it never stores request text, Discord IDs, destination IDs, tokens, secrets, message content, callback commands, or logs.
- `BU-0218`: Repeating the same origin registration is idempotent; changing an existing registration is rejected.
- `BU-0767`: Reading back a task's registered callback origin re-validates its schema version and re-checks the profile name and correlation ID against the same format rules enforced at registration time — a stored origin is never trusted merely because it exists on disk.
- `BU-0781`: Registering a callback origin for a task first validates that the named profile is an installed, safely-permissioned executable and that the correlation ID is well-formed, before any origin record is written.
- `BU-0782`: Registering a callback origin for a task that already has one registered is a silent no-op if the new registration is identical to the existing one, and a hard failure if it differs — a task's callback origin is pinned once and cannot be silently changed thereafter.

