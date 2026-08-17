# 40-instrument: instrument

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-hypothesize/output/README.md | L4 | upstream artifact produced by `30-hypothesize` |

## Purpose

One probe per prediction, one variable at a time, tagged logs.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

One probe per prediction, one variable at a time, tagged logs.

## Behavior contract

- **Each instrumentation probe in Phase 4 must map to a specific prediction from Phase 3, changing exactly one variable at a time.**
  (trigger: hypotheses exist from Phase 3; outcome: instrumentation is targeted and interpretable rather than diffuse)
- **Instrumentation tool preference is ordered: debugger/REPL inspection where the environment supports it (one breakpoint beats ten logs), then targeted logs at boundaries that distinguish hypotheses, and never 'log everything and grep'.**
  (trigger: choosing an instrumentation method; outcome: the least diffuse tool available is chosen first)
- **Every debug log must be tagged with a unique prefix (e.g. `[DEBUG-a4f2]`) so cleanup at the end becomes a single grep; untagged logs survive cleanup, tagged ones die.**
  (trigger: adding temporary debug logging; outcome: all temporary logging is grep-removable by construction)
- **For performance regressions, logs are usually the wrong tool; instead the actor establishes a baseline measurement (timing harness, `performance.now()`, profiler, query plan) and then bisects — measure first, fix second.**
  (trigger: diagnosing a performance regression specifically; outcome: a measured baseline exists before any fix is attempted)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Which instrumentation tool to reach for first, in the ordered preference (debugger/REPL, then targeted logs), and how to tag it.
- For performance regressions specifically, establishing a baseline measurement before bisecting.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when each prediction from `30-hypothesize` has its own tagged probe, one variable changed at a time.

### Decision evidence
The tagged probes and their results are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
