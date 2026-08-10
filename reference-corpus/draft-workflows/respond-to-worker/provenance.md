# Provenance — Respond to Worker

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W10** `respond-to-worker`.

## Stages

### `00-precondition-check`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-079` | Before responding to a worker, the operator must: read the exact finding/question and recommendation; ask only for missing product, risk, security, privacy, destructive, or irreversible decisions; record the decision in the owning td task; verify no unconsumed response generation already exists; and after sending, require the matching worker to acknowledge/consume it. | `reference/sergeant-upstream/docs/using-sergeant.md` (L253-262 (Respond to a worker)) |

### `10-validate-target`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-078` | A response can only ever be published against a worker whose current status is needs_input, blocked, waiting, or orphaned — any other status refuses the response outright, so a response is never silently applied to a worker that was not actually asking for one. | `reference/sergeant-upstream/bin/sgt-respond` (L202-205) |
| `BU-P7-060` | Publishing a response requires verifying worker identity and ownership evidence (session identity, worktree pointer/directory) recorded at dispatch time before the response is written, so a response can never be delivered to the wrong worker or a worktree Sergeant no longer actually owns. | `reference/sergeant-upstream/tests/sgt-respond-test.sh` (lines 9-46) |

### `20-publish-response`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-058` | Publishing a response must still durably store a delivered response even while a project or global drain is active, but must hold the relaunch of a stalled worker until the drain is lifted — admission control gates only the relaunch action, never the response storage itself. | `reference/sergeant-upstream/tests/sgt-respond-drain-test.sh` (lines 1-3) |
| `BU-P7-035` | `sgt-respond` must publish a response with no response-lock artifact left over on success, on immediate abort (mktemp failure), and on recovery from an empty, dead-PID, or stale-symlink leftover lock — but must fail immediately and actionably ("Response lock has an invalid owner") without touching the pending response when the lock file is not a recognizable lock shape at all. | `reference/sergeant-upstream/tests/runtime-bash-test.sh` (lines 84-172) |

### `30-deliver-and-accept`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-114` | A worker's readiness gate for delivering a notification is bounded, not infinite: it waits at most a fixed timeout per notification target, and on timeout reports the unreachable state exactly once as durable, nonce-scoped evidence plus a recoverable needs_input gate — it never fabricates acknowledgement, acceptance, delivery, completion, or an action lease. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L378-386) |
| `BU-P7-109` | The full durable notification handshake (nudge delivered, ack token written, acceptance confirmed, instruction followed exactly once, completion published) must be exercised end-to-end for EVERY harness in the shared registry, twice — once for the initial notification and once for a response notification delivered to a relaunched worker — because a prior test iterated harnesses but never actually reached the handshake files for any harness but one, letting a defect go unnoticed for every other harness. | `reference/sergeant-upstream/tests/sgt-worker-handshake-test.sh` (lines 1-15) |
| `BU-P7-059` | Response delivery must never leave a response indefinitely pending merely because delivery to a live worker session exceeded its bounded acknowledgement timeout; rerunning the identical command is the documented bounded-recovery path, performing exactly one worker relaunch and retiring the unresponsive original worker only after the replacement is validated. | `reference/sergeant-upstream/tests/sgt-respond-recovery-test.sh` (lines 1-13) |

### `40-apply-and-acknowledge`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-032` | A response can only be acknowledged when it is the exact pending response — matching response ID and a well-formed positive gate generation number — so an acknowledgement can never accidentally consume a different, superseding response. | `reference/sergeant-upstream/bin/sgt-ack-response` (L45-49) |
| `BU-P6-034` | An acknowledged response's terminal outcome must be internally consistent: a status of done requires a non-empty result already present, and a status of failed requires a non-blank reason string, or the acknowledgement is refused. | `reference/sergeant-upstream/bin/sgt-ack-response` (L88-94) |
| `BU-P7-041` | Acknowledging a response must verify the caller-provided response ID matches the pending response, the requesting execution context's identity matches the recorded worker identity, and the worker's post-application status/proof file is present and valid — each check refusing (and leaving the pending response untouched) before any archive or acknowledgement state is published. | `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 37-59) |
| `BU-P7-044` | An archived acknowledgement record with an empty (unset) applied-status field must not be treated as matching a proof file with no status= line — this specific empty-vs-empty comparison used to be silently accepted as an already-converged replay, which is a false 'already delivered' the finalizer must refuse. | `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 321-347) |

### `50-archive-evidence`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-042` | A successfully acknowledged response is archived (body, gate_generation, applied_status, proof) under a mode-700 directory with a mode-600 body file, and the archive's recorded gate_generation is fixed at acknowledgement time — later changes to the live response_generation counter must not retroactively alter it. | `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 100-110) |
| `BU-P7-052` | The single response-lock-protected action-lease finalizer must be a no-op success when there was never a notification or never an accepted lease, must record neither a spurious completion nor a spurious pending outcome in those cases, and must never fabricate a completion that the agent itself never durably proved. | `reference/sergeant-upstream/tests/sgt-lease-finalizer-test.sh` (lines 1-13) |
| `BU-P7-053` | A completed turn's finalization must be atomic: it publishes the completion record and writes a finalization record together, must not leave a pending marker behind, and must not leak the response lock it acquired to do so. | `reference/sergeant-upstream/tests/sgt-lease-finalizer-test.sh` (lines 94-99) |

### `60-notify-coordinator`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-027` | A worker's free-text update message is classified into exactly one durable event kind — completion (done*/failed*), escalation (needs_input*/blocked*), or a generic update — purely by matching the message's leading token, and that classification, not the raw text, is what becomes the durable record. | `reference/sergeant-upstream/bin/sgt-notify` (L31-36) |
| `BU-P6-030` | A worker completion or escalation notification is also written as a durable wiki activity entry distinguishing the completion/escalation heading and, when present, extracting and linking any GitHub PR URL mentioned in the message. | `reference/sergeant-upstream/bin/sgt-notify` (L111-124) |
| `BU-P7-047` | A worker's escalation notification is delivered as a durable, mode-600 marker file tagged `event=escalation`, and never exposes the message body in that marker; it is separately mirrored into the wiki activity log under a distinct 'Agent Escalation' label so a nonterminal escalation is never mislabeled as a completion. | `reference/sergeant-upstream/tests/sgt-notify-test.sh` (lines 30-44) |
| `BU-P7-048` | A `done:`-prefixed notification is classified and logged as an 'Agent Completion' event distinct from an escalation, and direct terminal-injection delivery is available only as an explicit backward-compatibility transport, never the default. | `reference/sergeant-upstream/tests/sgt-notify-test.sh` (line 55) |

### `70-relaunch-if-needed`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-079` | An outstanding notification action-lease from the worker being responded to is first attempted to converge through the one shared finalizer, using only the agent's own exact completion proof; only if that convergence fails does responding refuse with a specific remediation pointing at the exact evidence path. | `reference/sergeant-upstream/bin/sgt-respond` (L417-435) |
| `BU-P6-080` | A response relaunch never allows a second, superseding worker instance to displace the first without preserving the first instance's superseded notification-target identity as evidence — and if that evidence would conflict with already-recorded evidence, the relaunch refuses outright rather than losing the older evidence. | `reference/sergeant-upstream/bin/sgt-respond` (L437-449) |

