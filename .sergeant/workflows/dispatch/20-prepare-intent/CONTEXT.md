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

## Bounded judgment

Apply `@@bounded-judgment`.

### J1 — local choices allowed
- Mechanical write order across fleet state and each selected worktree — the content is already decided, only the sequencing of writing it is local.

### J5 — governing constraint
- **One canonical `.sergeant-intent.md` revision is written identically to fleet state and every selected work surface** (`BU-P8-059`) — no actor discretion changes which revision is canonical.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the one canonical intent revision exists and is written identically everywhere it must land.

### Decision evidence
The written intent revision is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
