# 20-prepare-intent: prepare intent

## Inputs

| File | Layer | Why |
|---|---|---|
| ../15-check-admission/output/README.md | L4 | upstream artifact produced by `15-check-admission` |

## Purpose

One canonical intent revision exists and is written identically to fleet state and every selected work surface.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

One canonical intent revision exists and is written identically to fleet state and every selected work surface.

## Behavior contract

- **Dispatching creates or reuses td work, creates isolated worktrees, writes worker briefs, and records fleet state; it writes the same .sergeant-intent.md revision into fleet state and every selected worktree, and that one artifact is treated as canonical for implementation decisions, reviews, PR text, successor/recovery work, and final validation.**
  (trigger: sgt-dispatch runs against one or more repos; outcome: every downstream actor and process for this dispatch (implementer, reviewer, recovery, final validation) reads the same single canonical intent revision)
  — `BU-P8-059`, `reference/sergeant-upstream/docs/using-sergeant.md` (L54-58 (Dispatch mode))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
