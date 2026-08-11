# 10-triage-and-route: pin scope, then triage and route

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure. N1 adjudication A4 folded `00-pin-scope` in ahead of this stage's own judgment: swapping the ref-fetch/base-pin implementation would leave this stage's checkpoint (a recorded, reproducible diff scope exists before triage) unchanged, so it runs first as a helper invocation.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

Refs are fetched and a fixed base commit pinned, with base SHA/commit list/diff scope recorded, before implementation; then the full originating context is read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure.

## Behavior contract

- **Routing work before implementation requires triage: read the full originating context, check for redundant/prior work, and classify into one of five categories (huge/foggy, hard bug or perf regression, uncertain design/UI, approved feature/fix, merge/rebase conflict), each of which loads a different canonical skill.**
  (trigger: a worker has read a task and must decide how to proceed; outcome: the worker enters exactly one of five known procedural branches instead of guessing a generic implementation path)
  — `BU-P7-007`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 2. Route the work')

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim. The helper invocation below runs first, mechanically, to establish the fixed scope this judgment then triages against.

## Additional note

This is the branching point that raises engine-gap **G6** (child-procedure invocation with its own checkpoints) — see `reference-corpus/synthesis.md` §5. It survives partially: representable today only by inlining the chosen discipline's stages, losing independent parent/child checkpoint and recovery visibility.

## Helper invocations (folded stages, N1 adjudication A4)

**1. pin scope** (formerly `00-pin-scope`) — refs fetched, a fixed base commit pinned, base SHA/commit list/diff scope recorded before implementation. Classified at extraction as deterministic machinery (§6.5) with no "Additional note" arguing otherwise; swapping the fetch/pin implementation leaves the checkpoint (a recorded, reproducible diff scope) unchanged.

- **A dispatched worker's mission brief pins a pre-implementation source of truth: fetch refs, pin a fixed base commit (normally the merge-base with origin/main), and record base SHA, commit list, and diff scope before implementation begins.**
  (trigger: a worker begins substantive implementation work; outcome: every later validation and review step operates against a recorded, reproducible diff scope rather than a moving target)
  — `BU-P7-005`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 1. Pin scope and source of truth')

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
