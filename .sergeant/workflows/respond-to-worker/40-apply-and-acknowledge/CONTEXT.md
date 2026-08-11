# 40-apply-and-acknowledge: validate target, publish response, deliver and accept, apply and acknowledge, archive evidence, notify coordinator, then relaunch if needed

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-precondition-check/output/README.md | L4 | upstream artifact produced by `00-precondition-check` |

## Purpose

Decision applied once, truthful status restored, applied id/generation/status recorded, then acknowledged from the owning context. N1 adjudication A4 folded six deterministic-machinery stages around this stage's own judgment: target validation, response publication, and delivery-with-acceptance run first as helper invocations; archiving, coordinator notification, and relaunch-if-needed run after, once the decision is applied and acknowledged. None of the six carried a judgment argument that survives §6.3's reimplementation test — swapping any one's implementation leaves its checkpoint unchanged.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

The target's status is verified respondable and its identity/ownership evidence checked; the response is durably stored (even under an active drain) before delivery; delivery is attempted through a bounded readiness gate, never fabricating acknowledgement on timeout; the decision is then applied exactly once, truthful status restored, applied id/generation/status recorded, and acknowledged from the owning context; the acknowledged response is archived atomically with its generation fixed at acknowledgement time; the coordinator is notified via exactly one classified durable event kind; and, if needed, convergence is attempted through the single finalizer before any relaunch refusal, with superseded identities preserved as evidence.

## Behavior contract

- **A response can only be acknowledged when it is the exact pending response — matching response ID and a well-formed positive gate generation number — so an acknowledgement can never accidentally consume a different, superseding response.**
  (trigger: a worker acknowledges a specific response by ID; outcome: an acknowledgement is bound to one exact response identity and generation, never a wildcard match)
  — `BU-P6-032`, `reference/sergeant-upstream/bin/sgt-ack-response` (L45-49)
- **An acknowledged response's terminal outcome must be internally consistent: a status of done requires a non-empty result already present, and a status of failed requires a non-blank reason string, or the acknowledgement is refused.**
  (trigger: a response is being acknowledged against a terminal worker status; outcome: a terminal status is never accepted as evidence without the substance (result or reason) that makes it a real terminal outcome)
  — `BU-P6-034`, `reference/sergeant-upstream/bin/sgt-ack-response` (L88-94)
- **Acknowledging a response must verify the caller-provided response ID matches the pending response, the requesting execution context's identity matches the recorded worker identity, and the worker's post-application status/proof file is present and valid — each check refusing (and leaving the pending response untouched) before any archive or acknowledgement state is published.**
  (trigger: sgt-ack-response is invoked to consume a delivered response; outcome: acknowledgement cannot be forged by the wrong execution context, the wrong response ID, or a fabricated proof; every validation failure leaves the original response fully intact for a correct retry)
  — `BU-P7-041`, `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 37-59)
- **An archived acknowledgement record with an empty (unset) applied-status field must not be treated as matching a proof file with no status= line — this specific empty-vs-empty comparison used to be silently accepted as an already-converged replay, which is a false 'already delivered' the finalizer must refuse.**
  (trigger: sgt-ack-response replays against a pre-existing archive entry; outcome: convergence/replay logic never accepts a degenerate empty-equals-empty comparison as proof of a genuinely completed acknowledgement)
  — `BU-P7-044`, `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 321-347)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim. The helper invocations below run before this judgment (reaching a delivered, acceptable response) and after it (recording and propagating the applied decision).

## Helper invocations (folded stages, N1 adjudication A4)

**1. validate target** (formerly `10-validate-target`) — the target's status is one of the four respondable states and its recorded identity/ownership evidence verifies; anything else refuses.

- **A response can only ever be published against a worker whose current status is needs_input, blocked, waiting, or orphaned — any other status refuses the response outright, so a response is never silently applied to a worker that was not actually asking for one.**
  (trigger: an operator supplies a response for a specific task/repo; outcome: responses are only ever delivered to workers in one of exactly four states that legitimately mean 'this worker is waiting for input')
  — `BU-P6-078`, `reference/sergeant-upstream/bin/sgt-respond` (L202-205)
- **Publishing a response requires verifying worker identity and ownership evidence (session identity, worktree pointer/directory) recorded at dispatch time before the response is written, so a response can never be delivered to the wrong worker or a worktree Sergeant no longer actually owns.**
  (trigger: a response is about to be published for a specific fleet task/repo; outcome: response delivery is bound to a durably recorded, ownership-verified worker identity rather than a bare task/repo name lookup)
  — `BU-P7-060`, `reference/sergeant-upstream/tests/sgt-respond-test.sh` (lines 9-46)

**2. publish response** (formerly `20-publish-response`) — the response is durably stored (even under an active drain) before any delivery is attempted.

- **Publishing a response must still durably store a delivered response even while a project or global drain is active, but must hold the relaunch of a stalled worker until the drain is lifted — admission control gates only the relaunch action, never the response storage itself.**
  (trigger: an operator responds to a needs_input worker while a drain is active; outcome: an operator's response is never lost merely because the fleet is draining; only the potentially-conflicting relaunch action is deferred)
  — `BU-P7-058`, `reference/sergeant-upstream/tests/sgt-respond-drain-test.sh` (lines 1-3)
- **`sgt-respond` must publish a response with no response-lock artifact left over on success, on immediate abort (mktemp failure), and on recovery from an empty, dead-PID, or stale-symlink leftover lock — but must fail immediately and actionably ("Response lock has an invalid owner") without touching the pending response when the lock file is not a recognizable lock shape at all.**
  (trigger: sgt-respond attempts to publish a response while an existing (possibly stale) response.lock is present; outcome: every recoverable lock shape (empty dir, dead PID, dangling symlink) converges to a clean publish, while an unrecognizable lock shape fails closed and preserves the original pending response untouched)
  — `BU-P7-035`, `reference/sergeant-upstream/tests/runtime-bash-test.sh` (lines 84-172)

**3. deliver and accept** (formerly `30-deliver-and-accept`) — bounded readiness gate; on timeout, a nonce-scoped unreachable record plus a recoverable gate — never a fabricated acknowledgement.

- **A worker's readiness gate for delivering a notification is bounded, not infinite: it waits at most a fixed timeout per notification target, and on timeout reports the unreachable state exactly once as durable, nonce-scoped evidence plus a recoverable needs_input gate — it never fabricates acknowledgement, acceptance, delivery, completion, or an action lease.**
  (trigger: a harness never becomes ready to receive a notification; outcome: an unreachable harness always surfaces as an actionable, recoverable needs_input state, never a misleading terminal orphaned status and never an infinite hang)
  — `BU-P6-114`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L378-386)
- **The full durable notification handshake (nudge delivered, ack token written, acceptance confirmed, instruction followed exactly once, completion published) must be exercised end-to-end for EVERY harness in the shared registry, twice — once for the initial notification and once for a response notification delivered to a relaunched worker — because a prior test iterated harnesses but never actually reached the handshake files for any harness but one, letting a defect go unnoticed for every other harness.**
  (trigger: any supported harness receives an initial or response notification; outcome: the durable handshake contract is proven identically for every harness the shared registry supports, not merely for the one harness earlier tests happened to cover deeply)
  — `BU-P7-109`, `reference/sergeant-upstream/tests/sgt-worker-handshake-test.sh` (lines 1-15)
- **Response delivery must never leave a response indefinitely pending merely because delivery to a live worker session exceeded its bounded acknowledgement timeout; rerunning the identical command is the documented bounded-recovery path, performing exactly one worker relaunch and retiring the unresponsive original worker only after the replacement is validated.**
  (trigger: a delivered response's acknowledgement timeout elapses with no ack from the worker; outcome: an operator has one deterministic, safe, idempotent next action (rerun the command) rather than being dead-ended between 'already pending' and 'not yet acknowledged' error states)
  — `BU-P7-059`, `reference/sergeant-upstream/tests/sgt-respond-recovery-test.sh` (lines 1-13)

**4. archive evidence** (formerly `50-archive-evidence`) — body, generation, applied status and proof archived atomically; the recorded generation is fixed at acknowledgement time.

- **A successfully acknowledged response is archived (body, gate_generation, applied_status, proof) under a mode-700 directory with a mode-600 body file, and the archive's recorded gate_generation is fixed at acknowledgement time — later changes to the live response_generation counter must not retroactively alter it.**
  (trigger: a response is acknowledged and archived; outcome: the archived record of what was approved and when is both access-restricted (private secrets) and immutable to later state changes, giving replay/audit a fixed fact)
  — `BU-P7-042`, `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 100-110)
- **The single response-lock-protected action-lease finalizer must be a no-op success when there was never a notification or never an accepted lease, must record neither a spurious completion nor a spurious pending outcome in those cases, and must never fabricate a completion that the agent itself never durably proved.**
  (trigger: any worker-exit or recycling path finalizes an accepted action lease; outcome: an action lease's terminal disposition is always an accurate, explicit, and singly-sourced record — never guessed, never silently dropped, never duplicated across two competing finalizer implementations)
  — `BU-P7-052`, `reference/sergeant-upstream/tests/sgt-lease-finalizer-test.sh` (lines 1-13)
- **A completed turn's finalization must be atomic: it publishes the completion record and writes a finalization record together, must not leave a pending marker behind, and must not leak the response lock it acquired to do so.**
  (trigger: a worker's turn completes with proof already published; outcome: finalization leaves exactly one consistent durable record (never both a pending and a finalized marker simultaneously) and releases its own lock)
  — `BU-P7-053`, `reference/sergeant-upstream/tests/sgt-lease-finalizer-test.sh` (lines 94-99)

**5. notify coordinator** (formerly `60-notify-coordinator`) — the update is classified into exactly one durable event kind and recorded; live transports are optional on top.

- **A worker's free-text update message is classified into exactly one durable event kind — completion (done*/failed*), escalation (needs_input*/blocked*), or a generic update — purely by matching the message's leading token, and that classification, not the raw text, is what becomes the durable record.**
  (trigger: a worker reports its status via sgt-notify; outcome: every notification is durably typed as completion, escalation, or update, independent of the message wording beyond its prefix)
  — `BU-P6-027`, `reference/sergeant-upstream/bin/sgt-notify` (L31-36)
- **A worker completion or escalation notification is also written as a durable wiki activity entry distinguishing the completion/escalation heading and, when present, extracting and linking any GitHub PR URL mentioned in the message.**
  (trigger: a worker update is being recorded; outcome: a durable, cross-referenced activity trail exists for every worker update independent of live delivery)
  — `BU-P6-030`, `reference/sergeant-upstream/bin/sgt-notify` (L111-124)
- **A worker's escalation notification is delivered as a durable, mode-600 marker file tagged `event=escalation`, and never exposes the message body in that marker; it is separately mirrored into the wiki activity log under a distinct 'Agent Escalation' label so a nonterminal escalation is never mislabeled as a completion.**
  (trigger: a worker publishes a needs_input escalation via sgt-notify; outcome: notification delivery is durable and private (secrets/message text never sit in a world-readable marker) while still being observable via a separate labeled activity trail)
  — `BU-P7-047`, `reference/sergeant-upstream/tests/sgt-notify-test.sh` (lines 30-44)
- **A `done:`-prefixed notification is classified and logged as an 'Agent Completion' event distinct from an escalation, and direct terminal-injection delivery is available only as an explicit backward-compatibility transport, never the default.**
  (trigger: sgt-notify is called with a done:-prefixed or explicit message; outcome: terminal and nonterminal notifications are classified differently by construction (message prefix), and the legacy direct-injection transport is opt-in only)
  — `BU-P7-048`, `reference/sergeant-upstream/tests/sgt-notify-test.sh` (line 55)

**6. relaunch if needed** (formerly `70-relaunch-if-needed`) — convergence attempted through the single finalizer before any refusal; superseded identities preserved as evidence.

- **An outstanding notification action-lease from the worker being responded to is first attempted to converge through the one shared finalizer, using only the agent's own exact completion proof; only if that convergence fails does responding refuse with a specific remediation pointing at the exact evidence path.**
  (trigger: a response relaunch would otherwise clear an outstanding action lease; outcome: a legitimate but unrecorded completion is never discarded by a relaunch, and a genuinely unfinished lease is refused with a concrete remediation, never silently overwritten)
  — `BU-P6-079`, `reference/sergeant-upstream/bin/sgt-respond` (L417-435)
- **A response relaunch never allows a second, superseding worker instance to displace the first without preserving the first instance's superseded notification-target identity as evidence — and if that evidence would conflict with already-recorded evidence, the relaunch refuses outright rather than losing the older evidence.**
  (trigger: a relaunch is superseding an existing notification target; outcome: the evidence trail for who was ever asked to act, and when they were superseded, is never lost or silently overwritten)
  — `BU-P6-080`, `reference/sergeant-upstream/bin/sgt-respond` (L437-449)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
