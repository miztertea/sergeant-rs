# 10-build-feedback-loop: build feedback loop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A named, already-run, red-capable, deterministic, fast, agent-runnable command exists, or the run stops and asks for access/artifacts.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

A named, already-run, red-capable, deterministic, fast, agent-runnable command exists, or the run stops and asks for access/artifacts.

## Behavior contract

- **A tight pass/fail signal that goes red specifically on the bug in question is the entire skill; with it, bisection/hypothesis-testing/instrumentation all just consume it, and without it no code-reading will substitute.**
  (trigger: starting Phase 1 (build a feedback loop); outcome: the actor commits to building a red-capable loop before any hypothesis work)
  — `BU-P2-021`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1 core principle, lines 14-14)
- **Disproportionate effort should be spent building the feedback loop; the actor should be aggressive, creative, and refuse to give up at this stage.**
  (trigger: building the feedback loop; outcome: effort is weighted toward loop construction over premature hypothesizing)
  — `BU-P2-022`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1, lines 16-16)
- **Feedback loops should be attempted in roughly this priority order: a failing test at the seam that reaches the bug; a curl/HTTP script against a running dev server; a CLI invocation diffed against a known-good snapshot; a headless-browser script (Playwright/Puppeteer); replaying a captured trace; a throwaway minimal harness; a property/fuzz loop for 'sometimes wrong output' bugs; a bisection harness automatable via `git bisect run`; a differential loop comparing old vs new versions or configs.**
  (trigger: no feedback loop exists yet; outcome: the actor selects a construction strategy from a ranked menu rather than inventing one ad hoc)
  — `BU-P2-023`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: loop-construction ladder items 1-9, lines 20-28)
- **Once a loop exists it should be tightened by asking whether it can be made faster (cache setup, skip unrelated init, narrow scope), sharper (assert on the specific symptom, not merely 'didn't crash'), and more deterministic (pin time, seed RNG, isolate filesystem, freeze network).**
  (trigger: a feedback loop exists but is slow/vague/flaky; outcome: the loop is iteratively improved along three named axes before being relied upon)
  — `BU-P2-025`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Tighten the loop, lines 35-39)
- **A 30-second flaky loop is barely better than no loop; a 2-second deterministic loop is treated as a debugging superpower.**
  (trigger: evaluating loop quality; outcome: loop quality is judged against a concrete speed/determinism bar)
  — `BU-P2-026`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Tighten the loop, lines 41-41)
- **For non-deterministic bugs the goal is a higher reproduction rate, not a clean repro: loop the trigger ~100x, parallelize, add stress, narrow timing windows, inject sleeps; a 50%-flake bug is debuggable, a 1% one is not, so the actor keeps raising the rate until it is debuggable.**
  (trigger: the bug does not reproduce deterministically; outcome: the actor drives reproduction rate up until the bug is debuggable, rather than treating flakiness as a dead end)
  — `BU-P2-027`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Non-deterministic bugs, lines 45-45)
- **If no loop can genuinely be built, the actor must stop and say so explicitly, list what was tried, and ask the user for environment access, a captured artifact (HAR, log dump, core dump, timestamped screen recording), or permission to add temporary production instrumentation — and must not proceed to hypothesize without a loop.**
  (trigger: every loop-construction attempt has failed; outcome: the actor escalates to the user with a specific, bounded ask instead of guessing)
  — `BU-P2-028`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: When you genuinely cannot build a loop, lines 49-49)
- **Phase 1 is complete only when the actor can name one already-run command (script path, test invocation, curl) that is red-capable (drives the actual bug path and asserts the user's exact symptom), deterministic (or, for flaky bugs, a pinned high reproduction rate), fast (seconds not minutes), and agent-runnable (unattended, human only via the HITL script).**
  (trigger: the actor believes a feedback loop is ready; outcome: phase completion is judged against four explicit, checkable criteria rather than a vibe)
  — `BU-P2-029`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Completion criterion, lines 53-58)
- **If the actor catches themselves reading code to build a theory before the red-capable command exists, that is the exact failure the skill prevents, and no red-capable command means no Phase 2.**
  (trigger: the actor is tempted to hypothesize before Phase 1 completes; outcome: the actor stops and returns to loop-building instead of hypothesizing early)
  — `BU-P2-030`, `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1 closing rule, lines 60-60)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Which construction strategy to attempt first from the ranked ladder, and how to tighten it along the three named axes (`BU-P2-023`, `BU-P2-025`).

### J1 — local choices allowed
- Local tooling/script naming within the chosen construction strategy.

### J0 — must become `needs_input`
- **No loop can genuinely be built.** Stop, list what was tried, and ask the user for environment access, a captured artifact, or permission to add temporary production instrumentation — never proceed to hypothesize without a loop (`BU-P2-028`, `BU-P2-030`).

### Completion boundary
This stage may complete only when the actor can name one already-run command that is red-capable, deterministic (or pinned high-rate for flaky bugs), fast, and agent-runnable (`BU-P2-029`).

### Decision evidence
The named command and its construction path are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
