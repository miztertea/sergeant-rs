# 60-reconcile: reconcile the repo set's completion facts

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-handoff-or-stop/output/README.md | L4 | upstream artifact produced by `50-handoff-or-stop` |

## Purpose

PR URLs, heads, CI, review threads, merge and deployment order, and terminal task/fleet state — **scoped to the specific repo set this Work's plan named** (N1 adjudication A8, BH-10). `dispatch` owns fleet-wide reconciliation (its automatic pre-launch sweep, cited `BU-P8-070`, folded under A4 into `dispatch/80-monitor`); this stage does not re-derive or invoke that procedure. It is an adjacent, owned procedure this stage names for the reader, not one it composes with — no `@@name` reference or delegation substitutes for a real child-workflow invocation, because none exists yet at this milestone (`docs/icm/convention.md` §4 rule 1; the underlying pressure to invoke it for real is recorded as evidence for engine-gap G6 in `reference-corpus/engine-pressure.md`, not as a new claim).

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

For the repositories named in this Work's plan only: PR URLs, heads, CI, review threads, merge and deployment order, and terminal task/fleet state for those repos. This stage does not assert or reconcile anything about repos, tasks, or fleet state outside its own plan's repo set — that is `dispatch`'s and `reconcile-and-cleanup-fleet`'s owned territory.

## Behavior contract

- **After dispatched workers finish, cross-repo-work reconciles PR URLs and final heads, required CI and unresolved review threads, merge order from dependency edges, deployment order and cross-repo release notes, and terminal td/fleet state and cleanup eligibility.**
  (trigger: dispatch has completed for the plan's repositories; outcome: the multi-repo outcome is reconciled against every planned gate, not just individual PR existence)
  — `BU-P5-052`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 87-93)
- **cross-repo-work never reports the cross-repo outcome complete until every owning repository has a terminal result or an explicit preserved blocker.**
  (trigger: reconciliation is being evaluated for completeness; outcome: partial completion is never reported as done)
  — `BU-P5-053`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 95-96)

**Scope note (N1 adjudication A8, BH-10):** `BU-P5-052`'s "terminal td/fleet state and cleanup eligibility" phrase is read here as reporting terminal state and cleanup eligibility *for the repos this plan named*, not as this stage owning fleet-wide reconciliation or cleanup. `dispatch` owns the automatic pre-launch fleet reconciliation sweep; `reconcile-and-cleanup-fleet` owns the actual cleanup decision and mutation once every repo is terminal. This stage only reports the repo-set-specific completion facts that feed those adjacent, owned procedures.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
