# 03-choose-mode: choose mode

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-load-context/output/README.md | L4 | upstream artifact produced by `01-load-context` |

## Purpose

Direct or dispatch is selected on the four stated criteria.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Direct or dispatch is selected on the four stated criteria.

## Behavior contract

- **Choose execution mode: direct for explicit single-repository work in this session; dispatch for cross-repository, parallel, or explicitly delegated work.**
  (trigger: queue checked; outcome: an execution mode is selected before further action)
  — `BU-P1-028`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L138, step 3)
- **Use dispatch mode when work spans repositories, contains two or more independent repository-owned tasks, needs an isolated independent review worker, or the user asks for workers.**
  (trigger: one of the four listed conditions holds; outcome: dispatch mode is selected as the executing procedure)
  — `BU-P1-003`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L15-17, dispatch-mode trigger)
- **Dispatch mode is used for cross-repository work, independent parallel repository tasks, isolated review workers, or an explicit request for workers; Sergeant creates isolated worktrees, injects repository instructions, and records fleet state.**
  (trigger: the trigger condition for dispatch mode holds; outcome: dispatch mode always creates isolated worktrees, injects instructions, and records fleet state)
  — `BU-P1-108`, `reference/sergeant-upstream/docs/what-is-sergeant.md` (docs/what-is-sergeant.md L68-72, Dispatch mode definition)
- **Direct mode is chosen when the user explicitly requests work in the current session and one repository owns the complete outcome.**
  (trigger: an operator or agent must decide between direct and dispatch mode for a new piece of work; outcome: the correct mode is chosen based on session/repo-ownership evidence, not guessed)
  — `BU-P8-053`, `reference/sergeant-upstream/docs/using-sergeant.md` (L18-19 (Direct mode))
- **Dispatch mode is chosen for cross-repository work, independent repository-owned tasks, isolated review workers, or an explicit request for workers.**
  (trigger: an operator or agent must decide between direct and dispatch mode for a new piece of work; outcome: dispatch is chosen when the outcome inherently needs isolation, multiple repos, or explicit worker delegation)
  — `BU-P8-054`, `reference/sergeant-upstream/docs/using-sergeant.md` (L30-33 (Dispatch mode))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helpers (folded per N1 adjudication A4)

`02-check-queue` carried no argument beyond the §6.5 deterministic-machinery boilerplate — no "Additional note" checkpoint argument — so it demotes by default and folds into this stage as a helper invocation performed before the mode decision:

- **Check queue.** Run sgt-td-list and reuse a matching task in direct or dispatch mode; create a task only when no canonical task exists.
  — `BU-P1-027`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L137, step 2)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
