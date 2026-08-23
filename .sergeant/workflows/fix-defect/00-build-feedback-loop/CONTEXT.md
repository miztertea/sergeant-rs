# 00-build-feedback-loop: build feedback loop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A named, already-run, red-capable, deterministic, fast, agent-runnable
command exists, or the run stops and asks for access/artifacts.

## What must become true here (durable outcome)

A named, already-run, red-capable, deterministic, fast, agent-runnable
command exists, or the run stops and asks for access/artifacts.

## Behavior contract

- **A tight pass/fail signal that goes red specifically on the bug in
  question is the entire skill; with it, hypothesis-testing and
  instrumentation all just consume it, and without it no code-reading
  will substitute.**
  (trigger: this stage begins; outcome: the actor commits to building a
  red-capable loop before any hypothesis work)
- **Disproportionate effort should be spent building the feedback loop;
  the actor should be aggressive, creative, and refuse to give up at this
  stage.**
  (trigger: building the feedback loop; outcome: effort is weighted
  toward loop construction over premature hypothesizing)
- **Feedback loops should be attempted in roughly this priority order: a
  failing test at the seam that reaches the bug; a curl/HTTP script
  against a running dev server; a CLI invocation diffed against a
  known-good snapshot; a headless-browser script; replaying a captured
  trace; a throwaway minimal harness; a property/fuzz loop for
  "sometimes wrong output" bugs; a bisection harness automatable via
  `git bisect run`; a differential loop comparing old vs. new versions
  or configs.**
  (trigger: no feedback loop exists yet; outcome: the actor selects a
  construction strategy from a ranked menu rather than inventing one ad
  hoc)
- **Once a loop exists it should be tightened by asking whether it can be
  made faster, sharper (assert on the specific symptom, not merely
  "didn't crash"), and more deterministic (pin time, seed RNG, isolate
  filesystem, freeze network).**
  (trigger: a feedback loop exists but is slow/vague/flaky; outcome: the
  loop is iteratively improved along three named axes before being relied
  upon)
- **For non-deterministic bugs the goal is a higher reproduction rate,
  not a clean repro: loop the trigger many times, parallelize, add
  stress, narrow timing windows, inject sleeps.**
  (trigger: the bug does not reproduce deterministically; outcome: the
  actor drives reproduction rate up until the bug is debuggable, rather
  than treating flakiness as a dead end)
- **If no loop can genuinely be built, the actor must stop and say so
  explicitly, list what was tried, and ask the user for environment
  access, a captured artifact, or permission to add temporary
  instrumentation — and must not proceed to hypothesize without a loop.**
  (trigger: every loop-construction attempt has failed; outcome: the
  actor escalates to the user with a specific, bounded ask instead of
  guessing)
- **This stage is complete only when the actor can name one already-run
  command that is red-capable, deterministic (or, for flaky bugs, a
  pinned high reproduction rate), fast, and agent-runnable.**
  (trigger: the actor believes a feedback loop is ready; outcome:
  completion is judged against four explicit, checkable criteria rather
  than a vibe)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Which construction strategy to attempt first from the ranked ladder, and
how to tighten it along the three named axes.

### J1 — local choices allowed
Local tooling/script naming within the chosen construction strategy.

### J0 — must become `needs_input`
No loop can genuinely be built. Stop, list what was tried, and ask the
user for environment access, a captured artifact, or permission to add
temporary instrumentation — never proceed to hypothesize without a loop.

### Completion boundary
This stage may complete only when the actor can name one already-run
command that is red-capable, deterministic (or pinned high-rate for
flaky bugs), fast, and agent-runnable.

### Decision evidence
The named command and its construction path are this stage's own
durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
