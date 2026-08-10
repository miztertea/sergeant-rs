# 00-require-terminal: require terminal, then reconcile and clean up

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Every targeted repo is safely terminal and the owning task is verifiably closed; "not closed" is distinguished from "could not be looked up". This is the sole judgment-bearing checkpoint of the workflow (N1 adjudication A4): every other behavior this package owns is deterministic machinery that crosses this checkpoint once it holds, folded in below as helper invocations rather than materialized as its own stage.

Trigger (workflow-level): A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## What must become true here (durable outcome)

Every targeted repo is safely terminal and the owning task is verifiably closed; ownership is re-verified by identity, handshake acknowledgement is confirmed and sealed, every repo's surface is removed without disturbing a live process, and whole-task state retires only once every repo is done — with "not closed" always distinguished from "could not be looked up".

## Behavior contract

- **Cleanup of a completed task is a bounded procedure that removes every repo's worktree (returning a treehouse lease or removing a plain git worktree) and, only when every repo is being cleaned at once, retires the fleet state directory itself — refusing to run at all unless every targeted repo's status is safely terminal, requiring the owning tracked-work task to be closed as well.**
  (trigger: a task's repos have all reached a safely terminal state and the operator wants to reclaim resources; outcome: worktrees are reclaimed and fleet state is retired only when every safety precondition (terminal status, closed tracked-work, response-handshake completeness) holds)
  — `BU-P6-135`, `reference/sergeant-upstream/bin/sgt-cleanup` (L2-7)
- **Cleanup refuses to remove a worktree unless the owning tracked-work task is verifiably closed — and it distinguishes 'not closed yet' from 'could not even be looked up', reporting the infrastructural failure by itself rather than letting an unreadable task tracker silently masquerade as 'not terminal'.**
  (trigger: cleanup is checking whether a repo's tracked work is done before removing its worktree; outcome: a diagnosability failure (couldn't check) is never confused with a real safety refusal (checked, and it's not closed))
  — `BU-P6-136`, `reference/sergeant-upstream/bin/sgt-cleanup` (L988-992, L1028-1042)
- **Fleet cleanup requires terminal/reconciled state, configured cleanup-owner proof for the repository/worktree or treehouse lease, preserved evidence, explicit cleanup-phase proof for a replayed removal or an already-absent worktree, fully acknowledged response transport, and no uncommitted or in-use worktree state; cleanup must never be used to resolve a waiting, blocked, or orphaned worker.**
  (trigger: sgt-cleanup is invoked for a fleet task; outcome: destructive cleanup only ever proceeds once every one of these conditions holds, and cleanup is never a shortcut for actually resolving unfinished work)
  — `BU-P8-092`, `reference/sergeant-upstream/docs/using-sergeant.md` (L399-408 (Clean completed fleet state))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocations (folded stages, N1 adjudication A4)

The six operations below were extracted as their own candidate stages
(ladder §6.5, "deterministic-machinery candidate") but carried no
judgment-bearing argument that survives §6.3's reimplementation test:
swapping each operation's implementation tomorrow would leave this
stage's checkpoint — safely terminal, ownership-verified, handshake-sealed,
surface removed, state retired — unchanged. They fold in here as ordered
helper invocations the acting harness performs (or invokes a script for)
once `00-require-terminal`'s own judgment call clears them to run. Order
matters and is preserved from extraction, with the two fleet-side checks
moved in from `monitor-fleet` under A7 sequenced first (they observe
before this stage mutates anything):

1. **reconcile terminal** (moved from `monitor-fleet/20-reconcile-terminal`, A7) — a `done` status with an empty result is refused as completion and marked orphaned; terminal recycling is identity-bound and settles the lease first.
   - **Fleet reconciliation recognizes a specific hazardous case — a status transitioning to done while the worktree's actual result file is empty — and refuses to accept it as a genuine completion, instead marking the Work orphaned with a diagnostic requiring a result before done can be trusted.**
     (trigger: a worker's status reads done but its recorded result is empty; outcome: a claimed completion is never trusted without the substantive evidence (a non-empty result) that makes it genuinely terminal)
     — `BU-P6-103`, `reference/sergeant-upstream/bin/sgt-watch` (L561-567)
   - **Retiring a terminal (done, failed, or drained) worker's durable session/execution identity's recycling evidence is bound to that exact identity, not merely stamped as a permanent task-level marker — because binding to the wrong scope (any prior recycling ever) permanently suppressed recycling of every later relaunched identity once one had ever been recycled.**
     (trigger: a terminal worker's execution identity needs to be recycled (its process resources reclaimed); outcome: every distinct execution identity a Work ever used gets recycled exactly once, even across multiple relaunches of the same Work)
     — `BU-P6-104`, `reference/sergeant-upstream/bin/sgt-watch` (L286-292)
   - **Recycling a terminal worker's execution identity first settles its accepted notification action-lease before that identity is torn down, because recycling used to stop the only process that could ever publish completion, which is exactly how a completed turn became permanently unrecoverable.**
     (trigger: a terminal worker's execution identity is about to be recycled; outcome: recycling never destroys the only process capable of proving a pending action was completed)
     — `BU-P6-105`, `reference/sergeant-upstream/bin/sgt-watch` (L322-326)
   - **Terminal-worker recycling must trigger for every terminal-adjacent status including `drained`, not only `done`/`failed:*`, and the recycling-suppression marker must be per-identity-bound and clearable, not a permanent task-level flag — a marker stamped merely because an identity went absent must not permanently suppress recycling of every later relaunched identity.**
     (trigger: reconciliation observes a fleet task reach a terminal-adjacent status; outcome: every terminal-adjacent status (including drained) is recycled exactly once per distinct identity, never permanently blocked by a stale marker from a prior identity)
     — `BU-P7-100`, `reference/sergeant-upstream/tests/sgt-watch-recycle-test.sh` (lines 5-11)
2. **verify ownership** (formerly `10-verify-ownership`) — repo identity, not path, is verified; retry-owner spoofing vectors are rejected.
   - **Cleanup never trusts a fleet-recorded worktree path as sufficient proof of ownership on its own; the resolved owning repository must be the exact same repository a previous pass recorded (verified by a repo identity, not just a path), and any recorded worktree that is present must independently be verified to belong to that owning repository, because a worktree replaced by an unrelated repository would otherwise answer lookups out of a foreign tracked-work database.**
     (trigger: cleanup needs to resolve which repository owns a fleet repo's tracked work; outcome: tracked-work status can never be looked up against the wrong repository just because a path happens to coincide)
     — `BU-P6-137`, `reference/sergeant-upstream/bin/sgt-cleanup` (L925-929, L956-959)
   - **Determining who legitimately owns a retry (whether the same repository is still the one recorded for a fleet task) must reject a wide range of repository-identity spoofing: symlink-aliased repos, same-origin clone replacement, independently-reset HEAD/refs, in-place repository or hook metadata changes, configured-worktree edits, repository replacement or move, and cross-project prefix-colliding or same-path repositories.**
     (trigger: cleanup or a related command re-verifies that a recorded repository is still the same one it was dispatched against; outcome: repository identity for retry/cleanup purposes cannot be spoofed by any of a wide, deliberately adversarial set of filesystem or git-state manipulations)
     — `BU-P7-081`, `reference/sergeant-upstream/tests/sgt-cleanup-test.sh` (line 731 (one of ~13 assert_retry_owner_rejected cases spanning lines 723-825))
3. **verify handshakes** (formerly `20-verify-handshakes`) — acknowledgement is verified, re-verified under lock immediately before deletion, and a terminal seal is written.
   - **Fleet cleanup for a task is blocked until sgt-callback check-acked succeeds for it, and immediately before the actual deletion, cleanup re-verifies the same condition under a callback lock and writes a terminal seal that rejects any new event generation, closing the race between the acknowledgement check and the deletion.**
     (trigger: cleanup is about to delete a task's fleet state; outcome: a task can never be deleted while any callback event is unacknowledged, and no new event can appear in the window between the check and the deletion)
     — `BU-P8-026`, `reference/sergeant-upstream/docs/callbacks.md` (L167-179)
   - **Rejected callback events are intentionally left unacknowledged and therefore also block cleanup until an operator repairs the consumer and reruns the retry command; and if cleanup fails after the terminal seal is written and the fleet must resume, only that specific seal (not any other state) may be removed, using an explicit unseal command.**
     (trigger: a callback event is in the reject state when cleanup is attempted, or a seal was written but cleanup then failed; outcome: a permanently-failed delivery cannot silently vanish through cleanup, and a stuck seal has one narrow, explicit recovery path)
     — `BU-P8-027`, `reference/sergeant-upstream/docs/callbacks.md` (L174-184)
4. **remove surface** (formerly `30-remove-surface`) — a resumable cleanup-phase record is published before and after; no process runs with its cwd inside the surface being removed.
   - **Removing a worktree publishes a durable, resumable cleanup-phase record before the removal begins and updates it again once removal completes, so that a cleanup interrupted mid-removal can be safely retried later: the retry re-verifies exact identity of every recorded fact (owner repo, worker evidence, worktree Git identity) rather than assuming the prior attempt's state is still accurate.**
     (trigger: cleanup is retried after a prior invocation was interrupted mid-worktree-removal; outcome: a retried cleanup can always resume exactly where an interrupted attempt left off, without either repeating destructive work unsafely or losing track of what already happened)
     — `BU-P6-140`, `reference/sergeant-upstream/bin/sgt-cleanup` (L2621-2642)
   - **Cleanup is only ever permitted to run when the worker's process cwd is verifiably not still inside the worktree being removed — verified via a system-wide process-working-directory scan (lsof) immediately before removal — so a worktree is never deleted while some process, tracked or untracked, still has it open.**
     (trigger: a worktree is about to be removed; outcome: a worktree removal can never proceed while any process — even one cleanup itself never launched or tracks — is still using it as its working directory)
     — `BU-P6-142`, `reference/sergeant-upstream/bin/sgt-cleanup` (L54-70, L268-270)
5. **retire state** (formerly `40-retire-state`) — whole-task state is retired only when every repo is cleaned together.
   - **Cleanup only ever retires the whole fleet-state directory for a task when every repo is being cleaned together (no repo filter given); a single-repo-scoped cleanup invocation only ever removes that one repo's worktree and never touches the shared task-level fleet state.**
     (trigger: cleanup is invoked, optionally scoped to a single repo; outcome: a task's shared fleet-level state (brief, intent, notifications) is only ever retired once every one of its repos has actually been cleaned up)
     — `BU-P6-141`, `reference/sergeant-upstream/bin/sgt-cleanup` (L2749-2751)
6. **background watch** (moved from `monitor-fleet/30-background-watch`, A7) — idempotent start, failed-start detection, stale-unit cleanup, graceful on unsupported platforms, for the process that keeps observing this task's fleet state after cleanup while any repo is not yet terminal. Already flagged at extraction (`synthesis.md`) as "closer to a deterministic helper for keeping the observation running than a procedural checkpoint" — its own note argues demotion, not survival, confirming the A4 disposition here.
   - **A background-watch invocation must be idempotent (a duplicate start is detected, not double-started), must detect and report a failed background start, must recognize and clean up a stale systemd unit, and must handle platforms without systemd support gracefully, in addition to covering ordinary active/terminal transitions.**
     (trigger: an operator or this stage starts background monitoring to persistently watch a fleet task toward terminal; outcome: background monitoring survives duplicate invocation, failed starts, stale leftover units, and TOCTOU races during cleanup, on platforms both with and without systemd)
     — `BU-P7-099`, `reference/sergeant-upstream/tests/sgt-watch-background-test.sh` (lines 1-4)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
