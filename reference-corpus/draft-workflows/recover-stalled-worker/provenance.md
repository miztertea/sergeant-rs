# Provenance — Recover Stalled Worker

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W11** `recover-stalled-worker`.

## Stages

### `00-collect-signals`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-095` | Diagnosing an in_progress-but-not-moving worker requires collecting four specific signals together — fleet status/log mtime, exact recorded process identity and its activity timestamp, fleet progress timestamp or current stall diagnostic, and td handoff plus current branch/worktree state — before any kill-or-relaunch decision, because a live parent process alone is insufficient evidence and a nonterminal stall diagnostic must still be reconciled through the documented progress rules first. | `reference/sergeant-upstream/docs/troubleshooting.md` (L52-68 (Worker says in_progress but is not moving)) |
| `BU-P8-099` | A repeated notification must be compared on task, repo, state generation, message digest, and timestamp before acting, because it can be a stale fleet record, an unconsumed response, or an incorrectly reclassified expected-blocked worker — and in no case should it produce a duplicate task or a duplicate response. | `reference/sergeant-upstream/docs/troubleshooting.md` (L96-100 (Repeated notifications)) |

### `10-preflight` (folded into `40-escalate-on-second-attempt`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-071` | A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch. | `reference/sergeant-upstream/bin/sgt-recover` (L6-10) |
| `BU-P6-073` | A recovery attempt refuses to proceed while an unfinished notification action-lease exists, unless the lease's owner is provably dead (its execution target no longer resolves at all and its recorded process is not running) — anything else, including a target that merely looks idle, must fail closed to preserve exact-once delivery evidence. | `reference/sergeant-upstream/bin/sgt-recover` (L140-155) |
| `BU-P6-075` | A recovery attempt runs every pre-flight validation — stall proof, lease convergence, drain check, relaunch-metadata completeness, prior-execution-instance identity — to completion before stamping the attempt as made, so that any pre-flight failure leaves the stalled worker untouched and eligible for a real recovery attempt later. | `reference/sergeant-upstream/bin/sgt-recover` (L229-232, L260-264) |
| `BU-P7-092` | Stall recovery (sgt-recover) must be refused while a drain is active, consistent with drain admission control blocking new relaunches — a stalled worker under an active drain is not relaunched by recovery. | `reference/sergeant-upstream/tests/sgt-recover-drain-test.sh` (line 2) |
| `BU-P7-093` | A missing `.sergeant-gate-generation` file must not leak a raw shell input-redirection error to stderr; and a pending action lease must not unconditionally refuse recovery — the lease owner's liveness and staleness must be adjudicated: a provably dead owner does not block recovery, while a live owner, a reused worker-session identifier, or an unprovable owner still fails closed. | `reference/sergeant-upstream/tests/sgt-recover-lease-owner-test.sh` (lines 1-14) |

### `20-launch-replacement` (folded into `40-escalate-on-second-attempt`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-072` | During recovery, the replacement worker is only launched, and its identity validated, before the original stalled worker instance is ever terminated, so that any failure in the relaunch sequence leaves the original stalled process intact for investigation rather than losing the supervisor entirely. | `reference/sergeant-upstream/bin/sgt-recover` (L12-15) |
| `BU-P7-094` | Recovery must validate a replacement supervisor's liveness, published identity, and notification-target creation BEFORE killing the stalled original — the kill must be strictly ordered after the replacement is confirmed live, and every abort path must restore fleet state so the recorded worker identity still points at the surviving original. | `reference/sergeant-upstream/tests/sgt-recover-replacement-test.sh` (lines 1-11) |

### `30-retire-original` (folded into `40-escalate-on-second-attempt`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-095` | Stall recovery performs exactly one bounded recovery attempt per invocation for an in-progress worker: terminate the stalled worker, relaunch a fresh worker, atomically update fleet metadata, and deliver a recovery notification — a single bounded operation, not an open-ended retry loop. | `reference/sergeant-upstream/tests/sgt-recover-test.sh` (line 2 and source-inventory row) |

### `40-escalate-on-second-attempt`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-071` | A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch. | `reference/sergeant-upstream/bin/sgt-recover` (L6-10) |

### `50-escalate-undocumented`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-109` | When documentation does not cover an observed failure, the operator should use the sergeant-help skill to search existing docs first, then create a td task containing the exact reproduction, expected behavior, preserved state, and acceptance criteria. | `reference/sergeant-upstream/docs/troubleshooting.md` (L242-244) |

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02):

| Stage | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|
| `10-preflight` | none | Swapping the preflight-validation implementation leaves the checkpoint — every precondition runs to completion before the attempt is stamped — unchanged. | **Demoted** — folded into `40-escalate-on-second-attempt` (first). |
| `20-launch-replacement` | none | Swapping the launch/validation implementation leaves the checkpoint — replacement proven live before original is touched — unchanged. | **Demoted** — folded into `40-escalate-on-second-attempt` (second). |
| `30-retire-original` | none | Swapping the retirement implementation leaves the checkpoint — original retired only after replacement proven live — unchanged. | **Demoted** — folded into `40-escalate-on-second-attempt` (third, immediately preceding its judgment). |
| `00-collect-signals`, `40-escalate-on-second-attempt`, `50-escalate-undocumented` | n/a (extracted as actor-stage, §6.4, already judgment-bearing) | n/a | **Kept** as extracted. |

Stage count: 6 extracted → 3 surviving. No behavior unit was deleted; all units cited under the three folded stages remain cited, now under `40-escalate-on-second-attempt`'s "Helper invocations" section (see that stage's `CONTEXT.md`). `BU-P6-071` is cited both at `40-escalate-on-second-attempt`'s own checkpoint and within the folded `10-preflight` content — the same fact serving both the gating and escalation halves of one invariant, not duplicated evidence.

