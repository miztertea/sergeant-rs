# validation-pipeline-gate — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a dispatched worker reaches readiness
- **Outcome:** the run only advances through an explicit pipeline-automation tool, never spontaneously
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `monitor-active-run`'s.

## How the stages relate

`validation-pipeline-gate` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain. The list below is the corpus's own defensible-default ordering (source/behavior_id order), not a claim that each stage's input is the previous stage's output.

1. `01-launch-validation-run` — exactly one validation-only boundary runs, in a split pane, with redundant stages skipped by default
2. `02-drive-gate-findings` — each disposition is handled by its own fixed rule, with ask-user always requiring a human decision
3. `03-recover-from-interrupted-run` — recovery follows exactly one of the three named branch_sync-driven paths
4. `04-declare-readiness` — readiness is durably recorded with intent/head/review evidence before the coordinator is notified, and the worker itself never invokes the validation pipeline
5. `05-acquire-launch-reservation` — exactly one validation launch proceeds per task/repository pair at a time, with concurrent attempts failing closed
6. `06-choose-intent-transport` — the default transport path never exposes intent content via process argv
7. `07-transfer-ownership` — ownership transfer requires cryptographic-strength process-ancestry proof of pane identity, and never displaces a live legitimate owner
8. `08-rollback-on-launch-failure` — rollback is scoped strictly to provably-owned artifacts of this invocation, never touching state it cannot prove it created
9. `09-verify-intent-consistency` — any divergence between the three intent copies, or between a recorded revision and the file's real hash, blocks validation rather than validating a possibly-stale or inconsistent intent
10. `10-reset-retryable-state` — a retry can never reset state while genuinely live validation processes, primary or unverified detached descendants, remain running
11. `11-create-isolated-snapshot` — the code actually validated is provably the exact reviewed commit, never a snapshot that silently drifted during creation
12. `12-check-coordinator-liveness` — a reused PID is never mistaken for the still-live original coordinator
13. `13-publish-worker-readiness-handshake` — the readiness handshake cannot be published, or later replayed by an unrelated process, without matching this exact revision+pane+pid+start-time tuple
14. `14-monitor-active-run` — the run only advances through an explicit pipeline-automation tool, never spontaneously

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

