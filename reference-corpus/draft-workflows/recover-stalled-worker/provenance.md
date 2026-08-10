# Provenance — Recover Stalled Worker

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W11** `recover-stalled-worker`.

## Stages

### `00-collect-signals`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-095` | Diagnosing an in_progress-but-not-moving worker requires collecting four specific signals together — fleet status/log mtime, exact recorded process identity and its activity timestamp, fleet progress timestamp or current stall diagnostic, and td handoff plus current branch/worktree state — before any kill-or-relaunch decision, because a live parent process alone is insufficient evidence and a nonterminal stall diagnostic must still be reconciled through the documented progress rules first. | `reference/sergeant-upstream/docs/troubleshooting.md` (L52-68 (Worker says in_progress but is not moving)) |
| `BU-P8-099` | A repeated notification must be compared on task, repo, state generation, message digest, and timestamp before acting, because it can be a stale fleet record, an unconsumed response, or an incorrectly reclassified expected-blocked worker — and in no case should it produce a duplicate task or a duplicate response. | `reference/sergeant-upstream/docs/troubleshooting.md` (L96-100 (Repeated notifications)) |

### `10-preflight`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-071` | A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch. | `reference/sergeant-upstream/bin/sgt-recover` (L6-10) |
| `BU-P6-073` | A recovery attempt refuses to proceed while an unfinished notification action-lease exists, unless the lease's owner is provably dead (its pane no longer resolves at all and its recorded process is not running) — anything else, including a pane that merely looks idle, must fail closed to preserve exact-once delivery evidence. | `reference/sergeant-upstream/bin/sgt-recover` (L140-155) |
| `BU-P6-075` | A recovery attempt runs every pre-flight validation — stall proof, lease convergence, drain check, relaunch-metadata completeness, old-pane identity — to completion before stamping the attempt as made, so that any pre-flight failure leaves the stalled worker untouched and eligible for a real recovery attempt later. | `reference/sergeant-upstream/bin/sgt-recover` (L229-232, L260-264) |
| `BU-P7-092` | Stall recovery (sgt-recover) must be refused while a drain is active, consistent with drain admission control blocking new relaunches — a stalled worker under an active drain is not relaunched by recovery. | `reference/sergeant-upstream/tests/sgt-recover-drain-test.sh` (line 2) |
| `BU-P7-093` | A missing `.sergeant-gate-generation` file must not leak a raw shell input-redirection error to stderr; and a pending action lease must not unconditionally refuse recovery — the lease owner's liveness and staleness must be adjudicated: a provably dead owner does not block recovery, while a live owner, a reused pane id, or an unprovable owner still fails closed. | `reference/sergeant-upstream/tests/sgt-recover-lease-owner-test.sh` (lines 1-14) |

### `20-launch-replacement`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-072` | During recovery, the replacement worker is only launched, and its identity validated, before the original stalled pane is ever killed, so that any failure in the relaunch sequence leaves the original stalled process intact for investigation rather than losing the supervisor entirely. | `reference/sergeant-upstream/bin/sgt-recover` (L12-15) |
| `BU-P7-094` | sgt-recover must validate a replacement supervisor's liveness, published identity, and notification-target creation BEFORE killing the stalled original — the kill must be strictly ordered after the replacement is confirmed live, and every abort path must restore fleet state so the recorded pane still points at the surviving original. | `reference/sergeant-upstream/tests/sgt-recover-replacement-test.sh` (lines 1-11) |

### `30-retire-original`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-095` | sgt-recover performs exactly one bounded stall-recovery attempt per invocation for an in-progress worker: kill the stalled pane, relaunch a fresh worker, atomically update fleet metadata, and deliver a recovery notification — matching the source inventory's description of a single bounded operation, not an open-ended retry loop. | `reference/sergeant-upstream/tests/sgt-recover-test.sh` (line 2 and source-inventory row) |

### `40-escalate-on-second-attempt`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-071` | A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch. | `reference/sergeant-upstream/bin/sgt-recover` (L6-10) |

### `50-escalate-undocumented`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-109` | When documentation does not cover an observed failure, the operator should use the sergeant-help skill to search existing docs first, then create a td task containing the exact reproduction, expected behavior, preserved state, and acceptance criteria. | `reference/sergeant-upstream/docs/troubleshooting.md` (L242-244) |

