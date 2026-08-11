# 60-task-tracking-init: task tracking init

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../40-repair-existing/output/README.md | L4 | upstream artifact produced by `40-repair-existing` |

## Purpose

Tracked-work storage initialized per registered repo, each behind explicit consent.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Tracked-work storage initialized per registered repo, each behind explicit consent.

## Behavior contract

- **For each registered repository, sergeant-setup checks td initialization via `td status --json --work-dir <repo-path>`; if not initialized, it shows and requires explicit consent for `td init --work-dir <repo-path>`, and reports the gap in the final summary if consent is declined.**
  (trigger: the project YAML lists repositories; outcome: every repository ends this phase either td-initialized, explicitly declined-and-reported, or already ok)
  — `BU-P5-032`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 235-248)
- **sergeant-setup never initializes td in a repository that is not registered in the current project YAML.**
  (trigger: Phase 7 iterates repositories for td initialization; outcome: td initialization stays scoped exactly to the project's declared repository set)
  — `BU-P5-033`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 250-251)
- **td initialization, Graphify initialization, and Treehouse initialization each require an explicit confirmation prompt before Sergeant performs them; none may be silently auto-initialized.**
  (trigger: setup encounters a repo lacking td, Graphify, or Treehouse configuration; outcome: no mutating initialization of an external tool happens without an explicit human yes)
  — `BU-P7-037`, `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 69-77)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helpers (folded per N1 adjudication A4)

`50-sync-and-verify` carried no argument beyond the §6.5 deterministic-machinery boilerplate — no "Additional note" checkpoint argument — so it demotes by default and folds into this stage as a helper invocation performed before per-repo task-tracking initialization begins:

- **Sync and verify.** After the project YAML is written, sergeant-setup runs sgt-list, sgt-context <project>, sgt-status <project>, and sgt-sync <project> in that fixed order, stopping at the first failure with its full output and never advancing to the next command until the previous one succeeds.
  — `BU-P5-031`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 220-231)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
