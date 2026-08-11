# 40-escalate-or-continue: escalate or continue, then publish result

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-independent-review/output/README.md | L4 | upstream artifact produced by `30-independent-review` |

## Purpose

A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete. N1 adjudication A4 folded `50-publish-result` in after this stage's own judgment: swapping the handoff-recording/readiness-wait implementation would leave this stage's checkpoint (an exactly-once, generation-safe handshake) unchanged, so it runs last as a helper invocation once the mission actually concludes rather than escalates.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete. If the mission instead concludes (rather than escalating), handoff evidence is recorded from the verified work surface, with readiness bounded and reported rather than hanging.

## Behavior contract

- **Every Sergeant notification must be acknowledged, then explicitly accepted by the supervisor, then acted on exactly once and marked complete — each step writing the same supervisor-scoped token to a distinct named file; repeated nudges carrying the same token are retries of the same action, never new work.**
  (trigger: the supervisor delivers a notification to a worker; outcome: a notification is durably ack'd, accepted, and completed exactly once, safe against duplicate delivery or supervisor restart)
  — `BU-P7-009`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume')
- **Before every new `needs_input` or `blocked` publication, a worker must increment a monotonic per-worktree gate-generation counter and persist it before writing the waiting status and message; a repeated blocker message is a new gate only when the generation actually advanced.**
  (trigger: a worker needs to publish a second (or later) blocking gate after a prior one was already resolved; outcome: each blocking gate is uniquely and monotonically identified, so a response can be proven to apply to the gate it was actually given for and not replayed against a stale one)
  — `BU-P7-012`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume')

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim. The helper invocation below runs after this judgment, mechanically, only on the path where the mission concludes rather than escalates.

## Additional note

The handshake is the durable part; the file-per-step mechanism that historically carried it is not (see obsolete-mechanism cluster M5).

## Helper invocations (folded stages, N1 adjudication A4)

**1. publish result** (formerly `50-publish-result`) — handoff evidence recorded from the verified work surface; readiness bounded and reported rather than hanging. Classified at extraction as deterministic machinery (§6.5) with no "Additional note" arguing otherwise; swapping the recording/readiness-wait implementation leaves the checkpoint (verified-worktree handoff evidence, bounded readiness) unchanged.

- **sgt-td-memory must record handoff evidence only from a verified worktree, and every git field it stores (branch, HEAD, etc.) must resolve from that specific worktree rather than from the supervisor's own current working directory — proven with two real linked worktrees on different branches/commits, not simulated.**
  (trigger: sgt-td-memory records recovery evidence for a worker; outcome: recorded recovery evidence (branch, commit, etc.) always describes the worker's actual worktree, never an ambient/wrong working directory, even under multi-worktree git setups)
  — `BU-P7-066`, `reference/sergeant-upstream/tests/sgt-td-memory-worktree-test.sh` (lines 1-18)
- **The interactive worker's wait for harness readiness must be bounded and its outcome reported — a harness that never renders must be caught and reported, not hang forever — and separately, a harness that becomes ready without ever acknowledging the notification must NOT be misrecorded as orphaned.**
  (trigger: a launched harness process may never become ready for input; outcome: the worker never spins forever waiting for readiness, and its eventual diagnosis distinguishes 'harness never became ready' from 'harness ready but never acknowledged' rather than conflating both into a generic orphaned status)
  — `BU-P7-110`, `reference/sergeant-upstream/tests/sgt-worker-readiness-test.sh` (lines 1-9)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
