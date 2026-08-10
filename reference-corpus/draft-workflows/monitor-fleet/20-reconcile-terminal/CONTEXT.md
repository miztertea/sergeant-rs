# 20-reconcile-terminal: reconcile terminal

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-evaluate-liveness/output/README.md | L4 | upstream artifact produced by `10-evaluate-liveness` |

## Purpose

A `done` status with an empty result is refused as completion and marked orphaned; terminal recycling is identity-bound and settles the lease first.

Trigger (workflow-level): An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## What must become true here (durable outcome)

A `done` status with an empty result is refused as completion and marked orphaned; terminal recycling is identity-bound and settles the lease first.

## Behavior contract

- **Fleet reconciliation recognizes a specific hazardous case — a status transitioning to done while the worktree's actual result file is empty — and refuses to accept it as a genuine completion, instead marking the Work orphaned with a diagnostic requiring a result before done can be trusted.**
  (trigger: a worker's status reads done but its recorded result is empty; outcome: a claimed completion is never trusted without the substantive evidence (a non-empty result) that makes it genuinely terminal)
  — `BU-P6-103`, `reference/sergeant-upstream/bin/sgt-watch` (L561-567)
- **Retiring a terminal (done, failed, or drained) worker's pane recycling evidence is bound to the exact pane identity being retired, not merely stamped as a permanent task-level marker — because binding to the wrong scope (any prior recycling ever) permanently suppressed recycling of every later relaunched pane once one pane had ever been recycled.**
  (trigger: a terminal worker's pane needs to be recycled (its process resources reclaimed); outcome: every distinct pane instance a Work ever used gets recycled exactly once, even across multiple relaunches of the same Work)
  — `BU-P6-104`, `reference/sergeant-upstream/bin/sgt-watch` (L286-292)
- **Recycling a terminal worker's pane first settles its accepted notification action-lease before the pane is taken away, because recycling used to stop the only process that could ever publish completion, which is exactly how a completed turn became permanently unrecoverable.**
  (trigger: a terminal worker's pane is about to be recycled; outcome: recycling never destroys the only process capable of proving a pending action was completed)
  — `BU-P6-105`, `reference/sergeant-upstream/bin/sgt-watch` (L322-326)
- **Terminal-worker recycling must trigger for every terminal-adjacent status including `drained`, not only `done`/`failed:*`, and the recycling-suppression marker must be per-pane/identity-bound and clearable, not a permanent task-level flag — a marker stamped merely because a pane went absent must not permanently suppress recycling of every later relaunched pane.**
  (trigger: sgt-watch observes a fleet task reach a terminal-adjacent status; outcome: every terminal-adjacent status (including drained) is recycled exactly once per distinct pane/identity, never permanently blocked by a stale marker from a prior pane)
  — `BU-P7-100`, `reference/sergeant-upstream/tests/sgt-watch-recycle-test.sh` (lines 5-11)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P6-104`, `BU-P6-105`, `BU-P7-100` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
