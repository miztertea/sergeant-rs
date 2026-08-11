# 01-build-feedback-loop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** diagnosing any hard bug

**Outcome:** effort concentrates on constructing the loop before anything else is attempted

**Statement (the operative rule):** A tight feedback loop that goes red specifically on this bug is the prerequisite for finding its cause; bisection, hypothesis-testing, and instrumentation only consume that signal, they do not replace it, and no amount of reading code substitutes for having one.

## What must become true here (durable outcome)

Effort concentrates on constructing the loop before anything else is attempted — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0943`: CONTEXT.md (if present) is read to build a mental model of the relevant modules, and ADRs in the touched area are checked, before further codebase exploration.
- `BU-0945`: As a last resort when no other feedback loop can be constructed, a human-in-the-loop bash script is used to drive the human through a structured repro loop instead of an ad hoc manual one, with the captured output fed back to the actor.
- `BU-0946`: Once a feedback loop exists, it is deliberately tightened along three dimensions: making it faster, making the signal sharper (assert on the specific symptom, not just 'didn't crash'), and making it more deterministic (pin time, seed RNG, isolate filesystem, freeze network).
- `BU-0947`: For a non-deterministic bug, the goal shifts from a single clean repro to a higher reproduction rate — looping the trigger repeatedly, parallelising, adding stress, narrowing timing windows, injecting sleeps — until the failure rate is high enough to debug against.
- `BU-0948`: If the actor genuinely cannot construct a feedback loop after trying, it stops and says so explicitly, lists what was tried, and asks the user for environment access, a captured artifact, or permission to add temporary production instrumentation — it does not proceed to hypothesise without a loop.
- `BU-0949`: Phase 1 is complete only when the actor can name one already-run command, with its pasted invocation and output, that is simultaneously red-capable (catches the user's exact symptom, not merely 'runs without erroring'), deterministic, fast, and agent-runnable (any human-in-the-loop step going only through the HITL script).
- `BU-0950`: If the actor notices itself reading code to build a theory before a red-capable command exists, it stops immediately — jumping to a hypothesis without a red-capable command is the exact failure this skill exists to prevent, and Phase 2 is not entered without one.

