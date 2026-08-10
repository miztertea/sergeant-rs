# 20-worker-side-checkpoint: worker side checkpoint

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-await-convergence/output/README.md | L4 | upstream artifact produced by `10-await-convergence` |

## Purpose

Idempotent drain detection; publish handoff and settle the lease before terminating anything.

Trigger (workflow-level): An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## What must become true here (durable outcome)

Idempotent drain detection; publish handoff and settle the lease before terminating anything.

## Behavior contract

- **A cooperative drain of one worker publishes every durable fact it can before terminating anything — a handoff, settlement of the outstanding action lease, and the drained status — and only after everything durable is published does it begin terminating processes, because a drain must never be a way to discard unfinished work.**
  (trigger: an active global or project drain is detected while a worker is running; outcome: a drained worker's true, honest state (never a fabricated result) is fully durable before any process is terminated)
  — `BU-P6-111`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L219-234)
- **A cooperative drain must actually terminate the worker's entire process group — not merely the backgrounded watcher subshell that detects the drain signal — and it must publish its durable handoff and finalize the action lease BEFORE terminating, leaving no pane and no surviving process behind.**
  (trigger: a project or global drain signals a worker to stop cooperatively; outcome: a worker marked 'drained' is actually and fully stopped — no live pane, no live agent process, no live background loop — with its handoff durably recorded first)
  — `BU-P7-084`, `reference/sergeant-upstream/tests/sgt-drain-terminate-test.sh` (lines 1-14)
- **A cooperative drain checkpoint inside the interactive worker must, on detecting drain, produce a clean exit with a `td` handoff written — including verifying the worktree it hands off from (per the same worktree-verification contract sgt-td-memory enforces elsewhere) — rather than exiting as if orphaned.**
  (trigger: a drained-status signal reaches a running interactive worker; outcome: a cooperatively drained worker leaves durable, worktree-verified recovery evidence behind, exactly like every other clean-exit path, rather than looking like an unexplained crash)
  — `BU-P7-107`, `reference/sergeant-upstream/tests/sgt-worker-drain-test.sh` (lines 20-25)
- **Cooperative drain detection inside the worker must be idempotent: an already-drained marker file present on disk must prevent a redundant re-drain, and it must distinguish global-drain, project-drain-match, project-drain-no-match, and no-drain-signal cases correctly, preserving all other worktree files across the drain transition.**
  (trigger: the worker evaluates whether it should cooperatively drain; outcome: drain detection is scoped precisely (global vs. this-project vs. other-project vs. none), preserves all worktree state, and never re-triggers drain handling once already drained)
  — `BU-P7-108`, `reference/sergeant-upstream/tests/sgt-drain-worker-test.sh` (lines 10-18)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P7-084` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
