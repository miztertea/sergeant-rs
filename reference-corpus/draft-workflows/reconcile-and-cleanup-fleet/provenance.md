# Provenance — Reconcile and Cleanup Fleet

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W15** `reconcile-and-cleanup-fleet`.

## Stages

### `00-require-terminal`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-135` | Cleanup of a completed task is a bounded procedure that removes every repo's worktree (returning a treehouse lease or removing a plain git worktree) and, only when every repo is being cleaned at once, retires the fleet state directory itself — refusing to run at all unless every targeted repo's status is safely terminal, requiring the owning tracked-work task to be closed as well. | `reference/sergeant-upstream/bin/sgt-cleanup` (L2-7) |
| `BU-P6-136` | Cleanup refuses to remove a worktree unless the owning tracked-work task is verifiably closed — and it distinguishes 'not closed yet' from 'could not even be looked up', reporting the infrastructural failure by itself rather than letting an unreadable task tracker silently masquerade as 'not terminal'. | `reference/sergeant-upstream/bin/sgt-cleanup` (L988-992, L1028-1042) |
| `BU-P8-092` | Fleet cleanup requires terminal/reconciled state, configured cleanup-owner proof for the repository/worktree or treehouse lease, preserved evidence, explicit cleanup-phase proof for a replayed removal or an already-absent worktree, fully acknowledged response transport, and no uncommitted or in-use worktree state; cleanup must never be used to resolve a waiting, blocked, or orphaned worker. | `reference/sergeant-upstream/docs/using-sergeant.md` (L399-408 (Clean completed fleet state)) |

### `10-verify-ownership`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-137` | Cleanup never trusts a fleet-recorded worktree path as sufficient proof of ownership on its own; the resolved owning repository must be the exact same repository a previous pass recorded (verified by a repo identity, not just a path), and any recorded worktree that is present must independently be verified to belong to that owning repository, because a worktree replaced by an unrelated repository would otherwise answer lookups out of a foreign tracked-work database. | `reference/sergeant-upstream/bin/sgt-cleanup` (L925-929, L956-959) |
| `BU-P7-081` | Determining who legitimately owns a retry (whether the same repository is still the one recorded for a fleet task) must reject a wide range of repository-identity spoofing: symlink-aliased repos, same-origin clone replacement, independently-reset HEAD/refs, in-place repository or hook metadata changes, configured-worktree edits, repository replacement or move, and cross-project prefix-colliding or same-path repositories. | `reference/sergeant-upstream/tests/sgt-cleanup-test.sh` (line 731 (one of ~13 assert_retry_owner_rejected cases spanning lines 723-825)) |

### `20-verify-handshakes`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-026` | Fleet cleanup for a task is blocked until sgt-callback check-acked succeeds for it, and immediately before the actual deletion, cleanup re-verifies the same condition under a callback lock and writes a terminal seal that rejects any new event generation, closing the race between the acknowledgement check and the deletion. | `reference/sergeant-upstream/docs/callbacks.md` (L167-179) |
| `BU-P8-027` | Rejected callback events are intentionally left unacknowledged and therefore also block cleanup until an operator repairs the consumer and reruns the retry command; and if cleanup fails after the terminal seal is written and the fleet must resume, only that specific seal (not any other state) may be removed, using an explicit unseal command. | `reference/sergeant-upstream/docs/callbacks.md` (L174-184) |

### `30-remove-surface`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-140` | Removing a worktree publishes a durable, resumable cleanup-phase record before the removal begins and updates it again once removal completes, so that a cleanup interrupted mid-removal can be safely retried later: the retry re-verifies exact identity of every recorded fact (owner repo, worker evidence, worktree Git identity) rather than assuming the prior attempt's state is still accurate. | `reference/sergeant-upstream/bin/sgt-cleanup` (L2621-2642) |
| `BU-P6-142` | Cleanup is only ever permitted to run when the worker's process cwd is verifiably not still inside the worktree being removed — verified via a system-wide process-working-directory scan (lsof) immediately before removal — so a worktree is never deleted while some process, tracked or untracked, still has it open. | `reference/sergeant-upstream/bin/sgt-cleanup` (L54-70, L268-270) |

### `40-retire-state`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-141` | Cleanup only ever retires the whole fleet-state directory for a task when every repo is being cleaned together (no repo filter given); a single-repo-scoped cleanup invocation only ever removes that one repo's worktree and never touches the shared task-level fleet state. | `reference/sergeant-upstream/bin/sgt-cleanup` (L2749-2751) |

