# 05-invoke-consumer

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a callback event is delivered to its consumer executable

**Outcome:** the consumer receives a minimized, argument-free, environment-scrubbed invocation surface

**Statement (the operative rule):** Sergeant invokes the fixed callback profile with no arguments, a minimal environment (HOME, PATH, locale, and temp-directory variables only), stderr discarded, and exactly one compact UTF-8 JSON object on stdin.

## What must become true here (durable outcome)

The consumer receives a minimized, argument-free, environment-scrubbed invocation surface — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0223`: The consumer must durably deduplicate by `idempotency_key` before creating a user-visible message, because a crash after external delivery but before acknowledgement causes an intentional retry.
- `BU-0224`: The callback executable has 15 seconds by default (`SGT_CALLBACK_TIMEOUT_SECONDS`, range 1-120) and may write at most 1024 bytes to stdout.
- `BU-0234`: The ws-lab hermes-discord consumer must forward the unchanged event through its source-bound forced transport, deduplicate durably by idempotency_key, map only the four event classes to bounded Discord text, return ack only after the fixed approved destination confirms delivery, and must never accept destination IDs or commands from any event field; Discord and Doppler credentials remain exclusively on the Hermes host and must not appear in the callback executable environment, stdout, stderr, or Sergeant fleet state.
- `BU-0790`: Invoking a callback profile passes it the event JSON on stdin and only a fixed allowlist of environment variables (HOME, PATH, LANG, LC_ALL, TMPDIR) — the invoking process's full environment, which may hold secrets, is never exposed to the callback subprocess.

