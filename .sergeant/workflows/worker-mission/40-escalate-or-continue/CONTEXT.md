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
- **Before every new `needs_input` or `blocked` publication, a worker must increment a monotonic per-worktree gate-generation counter and persist it before writing the waiting status and message; a repeated blocker message is a new gate only when the generation actually advanced.**
  (trigger: a worker needs to publish a second (or later) blocking gate after a prior one was already resolved; outcome: each blocking gate is uniquely and monotonically identified, so a response can be proven to apply to the gate it was actually given for and not replayed against a stale one)

## Bounded judgment

Apply `@@bounded-judgment`. The helper invocation below runs after this judgment, mechanically, only on the path where the mission concludes rather than escalates.

### J2 — delegated to this stage
- Whether the mission escalates (publishes a new gate) or concludes, based on the current state.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage — the escalate/continue handshake and gate-generation ordering are J5 governing constraints, not choices this stage exercises judgment over.

### Completion boundary
This stage may complete only when the handshake (ack/accept/act-once/complete) is fully recorded for an escalation, or handoff evidence is recorded from the verified worktree for a conclusion — never left partially written.

### Decision evidence
The token file per handshake step, or the recorded handoff evidence, is this stage's own decision record.

## Additional note

The handshake is the durable part; the file-per-step mechanism that historically carried it is not (see obsolete-mechanism cluster M5).

## Helper invocations (folded stages, N1 adjudication A4)

**1. publish result** (formerly `50-publish-result`) — handoff evidence recorded from the verified work surface; readiness bounded and reported rather than hanging. Classified at extraction as deterministic machinery (§6.5) with no "Additional note" arguing otherwise; swapping the recording/readiness-wait implementation leaves the checkpoint (verified-worktree handoff evidence, bounded readiness) unchanged.

- **sgt-td-memory must record handoff evidence only from a verified worktree, and every git field it stores (branch, HEAD, etc.) must resolve from that specific worktree rather than from the supervisor's own current working directory — proven with two real linked worktrees on different branches/commits, not simulated.**
  (trigger: sgt-td-memory records recovery evidence for a worker; outcome: recorded recovery evidence (branch, commit, etc.) always describes the worker's actual worktree, never an ambient/wrong working directory, even under multi-worktree git setups)
- **The interactive worker's wait for harness readiness must be bounded and its outcome reported — a harness that never renders must be caught and reported, not hang forever — and separately, a harness that becomes ready without ever acknowledging the notification must NOT be misrecorded as orphaned.**
  (trigger: a launched harness process may never become ready for input; outcome: the worker never spins forever waiting for readiness, and its eventual diagnosis distinguishes 'harness never became ready' from 'harness ready but never acknowledged' rather than conflating both into a generic orphaned status)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
