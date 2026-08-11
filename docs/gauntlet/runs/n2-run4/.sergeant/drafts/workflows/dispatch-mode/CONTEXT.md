# dispatch-mode — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** dispatch mode has been selected
- **Outcome:** a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `detect-model-substitution`'s.

## How the stages relate

`dispatch-mode` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain. The list below is the corpus's own defensible-default ordering (source/behavior_id order), not a claim that each stage's input is the previous stage's output.

1. `01-plan-and-decompose` — work is decomposed per-repository prior to dispatch
2. `02-dispatch-one-worker-per-repo` — each owning repository receives one dispatched worker via the dispatch step
3. `03-monitor-and-reconcile` — merge order, PR state, and cross-repo implications are reconciled by the coordinator
4. `04-validate-harness-selection` — only a recognized persistent-interactive agent selection is accepted; anything else is rejected before worker state is created
5. `05-resolve-and-record-model-pin` — the model is resolved deterministically by explicit precedence, and unpinned dispatches are explicitly recorded as such
6. `06-bind-and-verify-coordinator-pane` — pane binding happens through exactly one of the two mutually exclusive paths, always verified live, without relaxing the persistent-interactive-worker requirement
7. `07-publish-canonical-intent` — a single canonical intent revision is durably recorded and shared identically across fleet state and every worktree
8. `08-validate-intent-file` — dispatch requires a validated intent file before any mutating dispatch action, and validation failures block before mutation
9. `09-prepare-worker-brief` — the worker's brief carries the full task tracker lifecycle instructions rather than a freeform mission with no task tracking
10. `10-reconcile-dispatch-results` — dependency-gate satisfaction is a separate, required condition for done, not implied by other evidence
11. `11-create-tasks-before-spawn` — task creation is all-or-nothing across the selected repos, with rollback on partial failure, before any worker is spawned
12. `12-rollback-coordinator-pane-on-abort` — cleanup is scoped precisely to what this invocation created, and covers every later abort point from the moment the pane is bound
13. `13-check-drain-admission` — a race between a concurrent drain and this dispatch's admission is closed by holding the lock across the critical window, and any ambiguous drain record blocks rather than admits
14. `14-acquire-worktree` — prior committed work is never silently overwritten or orphaned by a fresh dispatch; resuming it requires an explicit --adopt-branch
15. `15-handle-spawn-failure` — every spawn failure path converges on the same explicit orphaning + evidence-recording sequence, never a silent or ambiguous half-started worker
16. `16-probe-harness-readiness` — a dead or still-blank pane is never reported ready
17. `17-capture-background-session-identity` — a malformed background ID can never be persisted into fleet state where every termination backstop assumes the well-formed shape
18. `18-reattach-after-attach-exit` — a legitimate cooperative gate (needs_input/blocked/waiting) is never mistaken for an unexpected death and spuriously re-attached
19. `19-detect-model-substitution` — a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

