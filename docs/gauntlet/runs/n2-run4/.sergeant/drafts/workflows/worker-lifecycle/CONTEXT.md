# worker-lifecycle — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a worker is resumed or recovered
- **Outcome:** the call refuses to stop anything and reports the inconsistency instead of guessing
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `stop-background-monitor`'s.

## How the stages relate

`worker-lifecycle` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain. The list below is the corpus's own defensible-default ordering (source/behavior_id order), not a claim that each stage's input is the previous stage's output.

1. `01-resume-model-pin-reverification` — the original model pin is honored exactly on resume, and an unhonorable tuple fails terminally instead of silently substituting a default
2. `02-drain-admission-lock` — locking either succeeds via hard link or the operation fails closed — it never proceeds without the lock
3. `03-deliver-mission` — delivery is exactly-once-safe across TUI startup delay or coordinator crash, and never exposes the mission body via process args
4. `04-bulk-reconcile-fleet-state` — panes are stopped only after identity verification, and ambiguous interrupted records wait out a grace period before being marked failed
5. `05-stalled-worker-recovery` — recovery is gated on having already reconciled identity/worktree/handoff/notification evidence, and only applies to the one named diagnostic
6. `06-recover-orphaned-worker` — orphaned is treated as requiring full reconciliation before any recovery action, not a quick retry
7. `07-enter-waiting-state` — the wait is represented durably (wake-condition file + waiting status) rather than a live sleep loop, permitting clean exit
8. `08-evaluate-and-resume-wait` — resumption happens only for the exact worker whose condition evaluates as met, tagged with a required generation
9. `09-drain-fleet-admission` — new pane admission is refused immediately; existing workers are allowed to finish cooperatively rather than being force-terminated on timeout
10. `10-respond-to-worker` — the five-step precondition/delivery sequence is followed before and after responding
11. `11-acknowledge-response` — sensitive transport is only cleared after private archival succeeds, and a retry after partial failure converges idempotently without re-applying the decision
12. `12-cleanup-fleet-task` — cleanup proceeds only once every named precondition holds, and never as a shortcut for a nonterminal worker state
13. `13-retire-response-handshake` — retirement requires two independently re-verified conditions (closed owning task, provably dead worker) on every attempt, not a one-time check
14. `14-seal-before-deletion` — a re-verified, locked, sealed check closes the race window between checking and actually deleting
15. `15-terminate-worker-process` — termination never signals processes outside the worker's own ownership just because they share an ambient process group id
16. `16-worker-exit-cleanup` — every background loop and the Claude background session are stopped on every exit path, including a clean completion no external script observes
17. `17-claim-action-lease` — at most one nonce ever holds the action lease for a given notification — a second target cannot silently steal or duplicate acceptance
18. `18-migrate-legacy-response-state` — migration only proceeds from a worktree whose identity is provably genuine
19. `19-relaunch-superseded-worker` — two Claude processes are never running concurrently against the same worktree
20. `20-force-stop-worker` — the command dies with an error if no matching drain is active
21. `21-recycle-terminal-worker-pane` — a relaunch that rebinds pane/pane_identity is correctly treated as needing its own recycling, rather than being permanently suppressed by an older marker
22. `22-classify-stalled-worker` — a worker that is actually producing tool-call or streamed output is never misclassified as stalled merely because progress_ts happens to be older
23. `23-reconcile-incomplete-dispatch` — evidence of committed work is surfaced to the operator rather than silently discarded behind a generic failure message
24. `24-stop-validation-pane` — a validation pane with incomplete ownership provenance is never terminated on an assumption
25. `25-start-background-monitor` — the call dies immediately with an actionable alternative rather than partially starting a monitor it cannot manage
26. `26-stop-background-monitor` — the call refuses to stop anything and reports the inconsistency instead of guessing

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

