# 30-instrument: instrument

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-hypothesize/output/README.md | L4 | the ranked hypotheses this stage gathers evidence against |

## Purpose

Evidence gathered against the hypothesis; the hypothesis is confirmed,
corrected, or replaced — attacking `20-hypothesize`'s output.

## What must become true here (durable outcome)

One probe per prediction, one variable at a time, tagged logs; each
hypothesis from `20-hypothesize` is confirmed, corrected, or replaced by
evidence, never promoted to cause without it.

## Behavior contract

- **Each instrumentation probe must map to a specific prediction from
  `20-hypothesize`, changing exactly one variable at a time.**
  (trigger: hypotheses exist; outcome: instrumentation is targeted and
  interpretable rather than diffuse)
- **Instrumentation tool preference is ordered: debugger/REPL inspection
  where the environment supports it, then targeted logs at boundaries
  that distinguish hypotheses, and never "log everything and grep".**
  (trigger: choosing an instrumentation method; outcome: the least
  diffuse tool available is chosen first)
- **Every debug log must be tagged with a unique prefix so cleanup at the
  end becomes a single grep; untagged logs survive cleanup, tagged ones
  die.**
  (trigger: adding temporary debug logging; outcome: all temporary
  logging is grep-removable by construction)
- **For performance regressions, logs are usually the wrong tool;
  instead the actor establishes a baseline measurement and then bisects —
  measure first, fix second.**
  (trigger: diagnosing a performance regression specifically; outcome: a
  measured baseline exists before any fix is attempted)
- **A hypothesis that survives instrumentation without evidence is
  recorded as unproven, not promoted to cause.**
  (trigger: instrumentation neither confirms nor disproves a hypothesis;
  outcome: the gap is recorded honestly rather than the hypothesis being
  treated as settled by elimination alone)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Which instrumentation tool to reach for first, in the ordered
  preference, and how to tag it.
- For performance regressions specifically, establishing a baseline
  measurement before bisecting.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
None specific to this stage beyond `@@bounded-judgment`'s general
triggers.

### Completion boundary
This stage may complete only when each prediction from `20-hypothesize`
has its own tagged probe, one variable changed at a time, and the
surviving hypothesis is stated as confirmed, corrected, or replaced.

### Decision evidence
The tagged probes and their results are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
