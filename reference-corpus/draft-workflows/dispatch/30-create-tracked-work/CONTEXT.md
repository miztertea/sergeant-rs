# 30-create-tracked-work: create tracked work

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-prepare-intent/output/README.md | L4 | upstream artifact produced by `20-prepare-intent` |

## Purpose

All-or-nothing task creation across every target repo, rolled back on any failure.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

All-or-nothing task creation across every target repo, rolled back on any failure.

## Behavior contract

- **Dispatching from a free-form brief creates exactly one td task per target repository before spawning any worker; if td is unavailable, task creation fails, generated metadata cannot be injected, or any selected repo fails to get a task, the whole dispatch aborts before spawning any worker and rolls back the generated tasks.**
  (trigger: a free-form dispatch is being launched; outcome: task creation for a multi-repo dispatch is all-or-nothing, never partially committed before any agent starts)
  — `BU-P5-088`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 196-198)
- **Creating tracked-work items across several repos for one cross-repo brief is all-or-nothing: every target repo is validated up front (cloned, task tracker initialized) before any task is created, and if creating a task in any repo fails after some were already created, every already-created task is deleted to roll back.**
  (trigger: a cross-repo brief needs one tracked-work item per target repo; outcome: either every target repo ends with exactly one new task, or none of them do — never a partial set left behind)
  — `BU-P6-036`, `reference/sergeant-upstream/bin/sgt-td-create` (L6-8, L192-193)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
