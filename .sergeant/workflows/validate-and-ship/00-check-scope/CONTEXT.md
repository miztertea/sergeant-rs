# 00-check-scope: check scope

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The invocation mode is determined and any specific user request is translated into concrete pipeline flags before anything runs.

Trigger (workflow-level, direct-invocation entry): the user invokes `/no-mistakes` directly in this session, with or without a task description.

## What must become true here (durable outcome)

The correct one of the two invocation modes (validate-only or task-first) is identified, and any specific user request (e.g. "skip the lint step") is translated into the matching pipeline flag rather than passed through unparsed.

## Behavior contract

- **no-mistakes has two invocation modes: validate-only, where the user's changes are already committed and the actor just validates and reports; and task-first, where the actor first carries out the described task, then validates the result.**
  (trigger: the user invokes /no-mistakes with or without a task description; outcome: the correct one of two distinct procedures is followed based on whether a task was given)
  — `BU-P2-059`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Two ways to invoke, lines 30-31)
- **When the user invokes /no-mistakes, the actor reports the outcome at the end; if the user asks for something specific (e.g. 'skip the lint step'), the actor translates that request into the matching `axi run` flag itself (e.g. `--skip=lint`), consulting `axi run --help` for available flags.**
  (trigger: the user invokes the no-mistakes command, optionally with a specific request; outcome: user intent is translated into concrete CLI flags rather than passed through unparsed)
  — `BU-P2-058`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (invocation, lines 20-23)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Distinguishing "already committed, just validate" (validate-only) from "do the task first" (task-first) from the user's own invocation (`BU-P2-059`).
- Translating an ambiguous natural-language request (e.g. "skip the lint step") into the correct `axi run` flag, consulting `axi run --help` rather than guessing the grammar (`BU-P2-058`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics — invocation-mode and flag-translation are the only material decisions this stage makes, and both are J2.

### J0 — must become `needs_input`
- The user's request cannot be mapped to any flag `axi run --help` actually offers.
- The invocation mode is genuinely ambiguous from what the user said (neither clearly "already committed" nor clearly "do this task first").

### Completion boundary
This stage may complete only when the correct invocation mode is identified and any specific user request is translated into a concrete, verified pipeline flag.

### Decision evidence
The chosen mode and flag translation carry forward as this stage's own durable output (`output/README.md`); no separate decision log.

## Additional note

Restored per N1 adjudication A5 (finding N1-BH-04): this checkpoint was previously dissolved into workflow-level citations and package notes rather than materialized, to avoid an id collision with what were then `10-acquire-launch-reservation`/`20-reserve-isolated-snapshot`. Adjudication A5 confirmed dissolving an extracted checkpoint to dodge a numbering collision is itself a violation — id collisions are resolved by renaming, never by dissolving the checkpoint. Those two stages have since been demoted to helpers under adjudication A4 (see `20-select-intent-transport`), which freed the `00`/`10` ordinals this stage and `10-do-the-work` now occupy.

This is the entry stage for the **directly-invoked** path (`/no-mistakes` run by the actor in the current session). The **coordinator-launched** path (a coordinator dispatching validation for a worker's already-reviewed commit) enters instead at `20-select-intent-transport`, whose folded helpers cover the readiness-marker, launch-reservation, and isolated-snapshot preconditions that only apply to that entry — see that stage's CONTEXT.md and this package's top-level CONTEXT.md "Relationships to other workflows" note. Both entries share every stage from `20-select-intent-transport` onward.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
