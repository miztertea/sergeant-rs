# 40-escalate-or-continue: escalate or continue

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-independent-review/output/README.md | L4 | upstream artifact produced by `30-independent-review` |

## Purpose

A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete.

## Behavior contract

- **Every Sergeant notification must be acknowledged, then explicitly accepted by the supervisor, then acted on exactly once and marked complete — each step writing the same supervisor-scoped token to a distinct named file; repeated nudges carrying the same token are retries of the same action, never new work.**
  (trigger: the supervisor delivers a notification to a worker pane; outcome: a notification is durably ack'd, accepted, and completed exactly once, safe against duplicate delivery or supervisor restart)
  — `BU-P7-009`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume')
- **Before every new `needs_input` or `blocked` publication, a worker must increment a monotonic per-worktree gate-generation counter and persist it before writing the waiting status and message; a repeated blocker message is a new gate only when the generation actually advanced.**
  (trigger: a worker needs to publish a second (or later) blocking gate after a prior one was already resolved; outcome: each blocking gate is uniquely and monotonically identified, so a response can be proven to apply to the gate it was actually given for and not replayed against a stale one)
  — `BU-P7-012`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume')

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

The handshake is the durable part; the file-per-step mechanism that historically carried it is not (see obsolete-mechanism cluster M5).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
