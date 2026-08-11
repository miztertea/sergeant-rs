# 04-enqueue-event

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the callback-delivery step enqueue is called

**Outcome:** the source identity is validated, never stored in plaintext, and re-use is idempotent rather than creating a duplicate event

**Statement (the operative rule):** The source ID for an enqueued event must match `^[a-z][a-z0-9._:-]{0,127}$` and is hashed before persistence; reusing the same event class/source ID returns the original event and generation.

## What must become true here (durable outcome)

The source identity is validated, never stored in plaintext, and re-use is idempotent rather than creating a duplicate event — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0221`: Callback payloads must be nonempty UTF-8, at most 4096 bytes and 16 lines, contain no NUL/control data other than tab/newline, and Sergeant rejects shell command metacharacters, command-like lines, secret-shaped assignments, and 17-20 digit platform IDs.
- `BU-0769`: Any callback event payload is capped at 4096 bytes, must decode as UTF-8, and — after a trailing newline is stripped — must be non-empty and no more than 16 lines, or it is rejected before being stored or delivered.
- `BU-0770`: A callback event payload is rejected if it contains raw control characters (other than newline/tab), shell/command metacharacters, a line that looks like an executable command invocation (shebang, bash/sh/curl/wget/ssh/sudo/rm/mv/cp/chmod/chown/python/node at line start), a secret-shaped key:value pattern (password/token/secret/api-key/etc.), or a long numeric platform identifier.
- `BU-0771`: Enqueuing a callback event for a source that has already produced an event of the same type is idempotent — the existing event is returned unchanged rather than a duplicate being created.
- `BU-0772`: A stored callback event's field set and schema version must match exactly, its generation must be a bounded positive integer, and the directory it is stored under must be named as the zero-padded form of that same generation, or it is rejected as an unsupported or inconsistent event.
- `BU-0773`: A stored callback event's correlation ID and profile must exactly match the task's registered origin; an event does not carry independent, potentially-diverged identity from the origin it belongs to.
- `BU-0774`: A stored callback event's idempotency key is not trusted from storage — it is recomputed from the event's correlation ID and generation and compared against the stored value, and a mismatch is rejected.
- `BU-0775`: Reading back a stored callback event re-validates its type against the known set, re-runs the same content-safety checks on its payload that were applied when it was first enqueued, and checks that its source hash and creation timestamp are well-formed.
- `BU-0778`: A new callback event's generation number is computed as one more than the higher of a durably persisted sequence counter and the highest generation number already present among that task's event directories, and the advanced counter is itself durably written before the generation is handed out.
- `BU-0780`: A newly enqueued callback event's event.json and state.json are written into a freshly created, privately-permissioned temporary directory and only made visible under their generation-numbered name via a single atomic directory rename, with the temporary directory's contents cleaned up on any failure.
- `BU-0813`: The enqueue CLI command refuses to enqueue a callback event for a task that has no registered callback origin.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0814`: The enqueue CLI command reads its payload from stdin bounded to one byte more than the maximum allowed payload size, so the size check itself can detect an over-limit read rather than silently truncating it.

