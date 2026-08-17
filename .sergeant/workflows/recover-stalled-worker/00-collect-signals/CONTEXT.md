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
- **A repeated notification must be compared on task, repo, state generation, message digest, and timestamp before acting, because it can be a stale fleet record, an unconsumed response, or an incorrectly reclassified expected-blocked worker — and in no case should it produce a duplicate task or a duplicate response.**
  (trigger: the same or a very similar notification arrives more than once; outcome: the operator investigates the specific cause rather than reflexively creating a new task or response for what may be a duplicate)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- No kill/relaunch decision on partial evidence — all four signals must be collected together first.
- Never produce a duplicate task or a duplicate response for a repeated notification.

### J2 — delegated to this stage
- Reconciling a nonterminal stall diagnostic through the documented progress rules.
- Investigating a repeated notification's specific cause — stale fleet record, unconsumed response, or misreclassified expected-blocked worker.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once all four signals are collected and any repeated notification is investigated to a specific cause.

### Decision evidence
The collected signals are this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
