# 00-load-project-context: load project context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Project context is loaded.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

Project context is loaded.

## Behavior contract

- **When loading project context for ticket authoring, do not automatically add td instructions to a repository's own guidance files as a side effect.**
  (trigger: project context is being loaded, and the repo lacks td instructions; outcome: the repository's own guidance files are left untouched unless explicitly requested)
  — `BU-P4-064`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Load Project Context, L46)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- Do not automatically add td instructions to a repository's own guidance files as a side effect (`BU-P4-064`).

### J2 — delegated to this stage
- None beyond the delegated context-loading itself, which carries its own bounded judgment.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once project context is loaded, without writing td instructions to any repository's own guidance files.

### Decision evidence
The loaded context is this stage's own durable output, recorded per `output/README.md`.

## Delegation

This stage's outcome is produced by running **estate-navigation** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet). Retargeted ICM-R3, 2026-08-16, from the retired `load-project` package (ABSORBED) — `estate-navigation`'s "Resolving estate context" section is the current analog.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
