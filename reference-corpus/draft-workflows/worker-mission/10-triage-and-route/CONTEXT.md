# 10-triage-and-route: triage and route

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-pin-scope/output/README.md | L4 | upstream artifact produced by `00-pin-scope` |

## Purpose

Full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

Full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure.

## Behavior contract

- **Routing work before implementation requires triage: read the full originating context, check for redundant/prior work, and classify into one of five categories (huge/foggy, hard bug or perf regression, uncertain design/UI, approved feature/fix, merge/rebase conflict), each of which loads a different canonical skill.**
  (trigger: a worker has read a task and must decide how to proceed; outcome: the worker enters exactly one of five known procedural branches instead of guessing a generic implementation path)
  — `BU-P7-007`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 2. Route the work')

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

This is the branching point that raises engine-gap **G6** (child-procedure invocation with its own checkpoints) — see `reference-corpus/synthesis.md` §5. It survives partially: representable today only by inlining the chosen discipline's stages, losing independent parent/child checkpoint and recovery visibility.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
