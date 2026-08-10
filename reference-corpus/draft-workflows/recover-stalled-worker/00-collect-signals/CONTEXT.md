# 00-collect-signals: collect signals

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Four signals are collected together before any kill/relaunch decision.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

Four signals are collected together before any kill/relaunch decision.

## Behavior contract

- **Diagnosing an in_progress-but-not-moving worker requires collecting four specific signals together — fleet status/log mtime, exact recorded process identity and its activity timestamp, fleet progress timestamp or current stall diagnostic, and td handoff plus current branch/worktree state — before any kill-or-relaunch decision, because a live parent process alone is insufficient evidence and a nonterminal stall diagnostic must still be reconciled through the documented progress rules first.**
  (trigger: a worker appears stuck at in_progress; outcome: a diagnosis is only trusted once all four signals are gathered, and no kill/relaunch happens on partial evidence)
  — `BU-P8-095`, `reference/sergeant-upstream/docs/troubleshooting.md` (L52-68 (Worker says in_progress but is not moving))
- **A repeated notification must be compared on task, repo, state generation, message digest, and timestamp before acting, because it can be a stale fleet record, an unconsumed response, or an incorrectly reclassified expected-blocked worker — and in no case should it produce a duplicate task or a duplicate response.**
  (trigger: the same or a very similar notification arrives more than once; outcome: the operator investigates the specific cause rather than reflexively creating a new task or response for what may be a duplicate)
  — `BU-P8-099`, `reference/sergeant-upstream/docs/troubleshooting.md` (L96-100 (Repeated notifications))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
