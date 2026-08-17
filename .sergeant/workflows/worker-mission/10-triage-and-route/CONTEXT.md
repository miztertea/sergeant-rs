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

## Bounded judgment

Apply `@@bounded-judgment`. The helper invocation below runs first, mechanically, to establish the fixed scope this judgment then triages against.

### J2 — delegated to this stage
- Classifying the pinned work into exactly one of the five named categories (huge/foggy, hard bug or perf regression, uncertain design/UI, approved feature/fix, merge/rebase conflict).

### J1 — local choices allowed
- None beyond ordinary tool mechanics — classification is the only material decision this stage makes, and it is J2.

### J0 — must become `needs_input`
- **The work straddles more than one of the five categories and no single classification is clearly dominant** (e.g. a hard bug that also requires an uncertain design call before it can be fixed). The five categories are mutually exclusive by construction, each loading a materially different procedure with a different authority envelope — classifying a straddling case is not the same delegation as choosing among genuinely exclusive categories, and guessing silently picks the wrong downstream procedure. (Parallel finding to `deepen-module`'s own structurally identical five-vs-four-category branching point, `sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r3/deepen-module/review.md`.)

### Completion boundary
This stage may complete only when scope is pinned and the work is classified into exactly one of the five categories (or raised as `needs_input` per the straddling case above).

### Decision evidence
The chosen category and its rationale are this stage's own durable output (`output/README.md`); no separate decision log.

## Additional note

This is the branching point that raises engine-gap **G6** (child-procedure invocation with its own checkpoints) — see `sergeant-rs-workspace/knowledge/evidence/reference-corpus/synthesis.md` §5. It survives partially: representable today only by inlining the chosen discipline's stages, losing independent parent/child checkpoint and recovery visibility.

## Helper invocations (folded stages, N1 adjudication A4)

**1. pin scope** (formerly `00-pin-scope`) — refs fetched, a fixed base commit pinned, base SHA/commit list/diff scope recorded before implementation. Classified at extraction as deterministic machinery (§6.5) with no "Additional note" arguing otherwise; swapping the fetch/pin implementation leaves the checkpoint (a recorded, reproducible diff scope) unchanged.

- **A dispatched worker's mission brief pins a pre-implementation source of truth: fetch refs, pin a fixed base commit (normally the merge-base with origin/main), and record base SHA, commit list, and diff scope before implementation begins.**
  (trigger: a worker begins substantive implementation work; outcome: every later validation and review step operates against a recorded, reproducible diff scope rather than a moving target)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
