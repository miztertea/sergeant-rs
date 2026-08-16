# 00-check-queue-and-plan: check queue and plan

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Either an existing tracked task supplies brief/branch/context, or a free-form brief plus explicit repo list is confirmed as accurate before anything is created.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Either an existing tracked task supplies brief/branch/context, or a free-form brief plus explicit repo list is confirmed as accurate before anything is created.

## Behavior contract

- **Before planning from scratch, dispatch checks whether the task already exists in td; if the user's request maps to an open td task, dispatch is invoked with that task id, and the brief, branch name, and full task context are pulled from td automatically, including instructions for the task lifecycle commands the worker must run.**
  (trigger: dispatch is about to plan a task; outcome: existing tracked work is reused rather than re-derived, and the worker's task-lifecycle obligations are pre-populated)
  — `BU-P5-057`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 27-36)
- **Before dispatching, dispatch states the plan explicitly -- which repos, what each does, dependency order, branch, and backend -- and requires that the plan be confirmed as accurate before proceeding.**
  (trigger: a dispatch plan (from a td task or a free-form brief) is ready; outcome: the operator sees and confirms the exact plan before any worktree or agent is created)
  — `BU-P5-058`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 38-56)
- **dispatch can be invoked from an existing td task via the dispatch command with an explicit task-id argument, which auto-detects the owning repo and derives the brief and branch from the task, optionally overriding the repo set explicitly.**
  (trigger: an open td task exists for the work; outcome: a dispatch can be launched from tracked-task identity alone, without re-stating the brief)
  — `BU-P5-059`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 58-68)
- **dispatch can also be invoked from a free-form brief with an explicit repository list, branch name, and dependency string, when no existing td task covers the work.**
  (trigger: no existing td task covers the work; outcome: a dispatch can still be launched from an ad hoc brief with explicit scope, branch, and dependencies)
  — `BU-P5-060`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 70-77)
- **Dispatch is a bounded, independently invocable procedure: given a project, a brief (or a tracked-work task reference), and a set of target repos, it produces one durable task with an isolated worktree, a rendered mission brief, and a spawned interactive worker per repo — with every side effect (tracked-work creation, worktree acquisition, worker-process launch) validated and gated before the next repo's dispatch begins.**
  (trigger: an operator or an automated caller (dagr hook) needs a piece of work executed by an agent in one or more repos; outcome: a durable, observable Work exists per targeted repo, each with its own isolated code snapshot and a running worker)
  — `BU-P6-123`, `reference/sergeant-upstream/bin/sgt-dispatch` (L1-5)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Deriving the plan (repos, dependency order, branch, backend) from a td task or a free-form brief (`BU-P5-057`, `BU-P5-059`, `BU-P5-060`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- The stated plan is not confirmed as accurate before proceeding — `BU-P5-058` requires explicit confirmation, not an inferred go-ahead.

### Completion boundary
This stage may complete only when either an existing tracked task supplies brief/branch/context, or a free-form brief plus explicit repo list is confirmed accurate.

### Decision evidence
The confirmed plan is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
