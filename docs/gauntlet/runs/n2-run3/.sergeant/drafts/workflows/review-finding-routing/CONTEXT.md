# Review Finding Routing

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A dispatched worker submits a review-finding artifact to sgt-review-findings.

**Outcome.** The finding is normalized, deduplicated, and routed into exactly one of four defined dispositions as an owning-repo td card, without ever silently overwriting a hand-edited card.

**Completion.** Route-finding, followed by reconcile-hand-edit on any rerun that meets a card modified outside the router.

## How its stages relate

Ordered, trigger-to-outcome:

1. **route-finding** (`01-route-finding/`) -- Trigger: a dispatched worker produces a review finding artifact. Outcome: actionable findings become owning-repo td tasks with durably published blocking guidance.
2. **reconcile-hand-edit** (`02-reconcile-hand-edit/`) -- Trigger: a stored finding card has been modified outside the router since it last wrote it. Outcome: the human-edited content is preserved (not overwritten) and flagged for human reconciliation.

## Workflow-local helper machinery (not separately packaged)

12 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.

## External shared dependencies (not part of this package)

- **5e shared-review-axis-definition** (`BU-0095, BU-0300`) -- finding acceptance/routing. Lives in `.sergeant/common/` once promoted; does not exist yet in this worktree, so this package cannot reference it by `@@name` (`../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` rule 5) and does not attempt to.
