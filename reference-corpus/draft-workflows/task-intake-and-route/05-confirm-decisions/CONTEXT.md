# 05-confirm-decisions: confirm decisions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-choose-mode/output/README.md | L4 | upstream artifact produced by `03-choose-mode` |

## Purpose

Only genuinely unresolved scope/risk decisions are put to the user.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Only genuinely unresolved scope/risk decisions are put to the user.

## Behavior contract

- **Confirm only unresolved decisions that change scope or risk: ask only when repository ownership, user-visible behavior, security/privacy policy, data retention, destructive action, or an irreversible tradeoff is unknown; do not ask the user to reconfirm an execution mode, plan, or tradeoff already recorded in the conversation or td.**
  (trigger: state reconciled; outcome: the user is asked only for genuinely missing, risk-changing decisions)
  — `BU-P1-030`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L140, step 5)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helpers (folded per N1 adjudication A4)

`04-reconcile-state` carried no argument beyond the §6.5 deterministic-machinery boilerplate — no "Additional note" checkpoint argument — so it demotes by default and folds into this stage as a helper invocation performed before deciding what remains genuinely unresolved:

- **Reconcile existing state.** Run sgt-watch --sync-all, then inspect active workers, branches, worktrees, retained gates, and handoffs before starting; resume or take over preserved work rather than creating duplicates.
  — `BU-P1-029`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L139, step 4)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
