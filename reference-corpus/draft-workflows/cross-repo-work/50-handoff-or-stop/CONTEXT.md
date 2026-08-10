# 50-handoff-or-stop: handoff or stop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-define-delivery-gates/output/README.md | L4 | upstream artifact produced by `40-define-delivery-gates` |

## Purpose

Either the plan is returned (planning-only) or control passes to dispatch; the coordinator never edits several repos itself.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

Either the plan is returned (planning-only) or control passes to dispatch; the coordinator never edits several repos itself.

## Behavior contract

- **If the user requested planning only, cross-repo-work stops after returning the briefs, acceptance evidence, and dependency graph, without dispatching or editing any repository; if implementation was requested, it hands off to the dispatch workflow via its launch command, and the primary session itself never edits several repositories directly.**
  (trigger: planning is complete; outcome: either the plan is returned for review, or execution is handed to a distinct dispatch procedure -- the planning session never becomes a multi-repo editor itself)
  — `BU-P5-051`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 79-85)
- **sgt-dispatch must never itself carry out `git checkout -b`, `git push -u origin`, or `gh pr create` as its own inline behavior in the cross-repo-work skill's prose; these operations belong to the dispatched worker, not to the coordinating skill.**
  (trigger: the coordinator is planning cross-repo work; outcome: the coordinating skill stays a planning/decomposition procedure and never performs the worker's own git mutations itself)
  — `BU-P7-017`, `reference/sergeant-upstream/tests/instruction-policy-test.sh` (lines 69-71)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **dispatch** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
