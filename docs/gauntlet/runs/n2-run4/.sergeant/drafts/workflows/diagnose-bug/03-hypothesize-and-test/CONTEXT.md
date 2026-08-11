# 03-hypothesize-and-test

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-reproduce-and-minimize/output/outcome.md | L4 | upstream evidence produced by `reproduce-and-minimize` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** beginning to hypothesise about a minimised, reproduced bug

**Outcome:** testing starts from a ranked set of candidates rather than the first idea

**Statement (the operative rule):** 3-5 ranked hypotheses are generated before any of them is tested, specifically to avoid anchoring on the first plausible idea.

## What must become true here (durable outcome)

Testing starts from a ranked set of candidates rather than the first idea — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0956`: Each hypothesis must be falsifiable — it must state the concrete prediction it makes ('if X is the cause, then changing Y will make the bug disappear / changing Z will make it worse').
- `BU-0957`: A hypothesis that cannot be phrased as a falsifiable prediction is discarded or sharpened rather than tested as-is, since an unfalsifiable hypothesis is treated as a vibe, not a real hypothesis.
- `BU-0958`: The ranked hypothesis list is shown to the user before testing begins, since domain knowledge can re-rank it or rule hypotheses out as already-known-false, but this checkpoint does not block: the actor proceeds with its own ranking if the user is AFK.
- `BU-0959`: Each Phase 4 instrumentation probe is tied to a specific prediction from Phase 3, and only one variable is changed at a time.
- `BU-0960`: The preferred instrumentation technique, in order, is a debugger/REPL breakpoint where the environment supports it, then targeted logs placed at the boundary that distinguishes hypotheses; 'log everything and grep' is never used.
- `BU-0961`: Every debug log added during Phase 4 is tagged with a unique prefix (e.g. `[DEBUG-a4f2]`) so cleanup at the end is a single grep — an untagged log would otherwise survive cleanup by accident.
- `BU-0962`: For a performance regression, logs are treated as usually the wrong tool; instead a baseline measurement (timing harness, profiler, or query plan) is established first and bisection is done second — measure first, fix second.

