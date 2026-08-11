# Task Intake And Execution

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A task is brought to a Sergeant session.

**Outcome.** The task reaches a durably recorded terminal/deliverable state through the mode-appropriate execution path, with evidence-preserving cleanup only after that state is verified.

**Completion.** Reconcile-and-deliver confirms terminal state and preserved evidence before any cleanup runs.

## How its stages relate

Ordered, trigger-to-outcome:

1. **resolve-context** (`01-resolve-context/`) -- Trigger: a task is brought to the session. Outcome: context is fully loaded before an execution mode is chosen.
2. **resolve-task** (`02-resolve-task/`) -- Trigger: context has been loaded. Outcome: an existing canonical td task is reused, or a new one created only otherwise.
3. **reconcile-before-start** (`03-reconcile-before-start/`) -- Trigger: an execution mode has been chosen, before starting work. Outcome: existing state is reconciled and reused rather than duplicated.
4. **execute** (`04-execute/`) -- Trigger: decisions have been confirmed. Outcome: execution proceeds via the mode-appropriate path.
5. **direct-mode-implementation** (`05-direct-mode-implementation/`) -- Trigger: direct mode is active. Outcome: the owning td task is claimed/created and implementation proceeds test-first.
6. **handle-decision-gate** (`06-handle-decision-gate/`) -- Trigger: a worker reaches needs_input, blocked, or an ask-user gate. Outcome: only genuinely missing decisions are solicited and recorded in td; remediation continues without redundant re-asks.
7. **direct-mode-delivery** (`07-direct-mode-delivery/`) -- Trigger: a direct-mode implementation is ready for delivery. Outcome: delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied.
8. **reconcile-and-deliver** (`08-reconcile-and-deliver/`) -- Trigger: work has reached a terminal or deliverable state. Outcome: cleanup runs only after terminal state and evidence preservation are verified.

## Unattached stage-context evidence, not materialized

2 `stage-context` behavior_id(s), across 2 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## External shared dependencies (not part of this package)

- **5a toolbelt-command-resolution** (`BU-0020, BU-0056`) -- invokes sgt-* commands. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.

## Note

**Ordering call, recorded plainly.** `execute` and `direct-mode-implementation` are both real `stage` classifications with overlapping territory -- `execute`'s trigger, "decisions have been confirmed," reads as the generic step from the standard workflow, while `direct-mode-implementation`'s trigger, "direct mode is active," is the same moment narrowed to direct mode specifically. They are kept as distinct, adjacent stages rather than merged, since the source classification records are separate and neither one's `rationale` treats the other as redundant (carried through from `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` #2).
