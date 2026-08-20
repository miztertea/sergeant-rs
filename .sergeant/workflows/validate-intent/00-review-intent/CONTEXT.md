# 00-review-intent: review intent

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Every one of the eight dimensions (AGENTS.md's Captain intent discipline)
is reported covered, gap, or not-applicable against the intent document
under review.

Trigger (workflow-level): Captain wants an intent document checked for
coverage before an expensive or dangerous dispatch — optional tooling,
invoked deliberately, never a required gate.

## What must become true here (durable outcome)

Every one of the eight dimensions is reported `covered`, `gap` (naming
what's missing), or `not-applicable` (with a reason); the intent itself
is never rewritten and no gap is ever filled with invented content.

## Behavior contract

- **The intent document under review is checked against exactly the eight dimensions AGENTS.md's `### INTENT — Captain's intent discipline` names (Objective, Required Invariants, Approved Tradeoffs, Out Of Scope, State Transitions, Failure Windows, Negative Test Matrix, Validation Evidence) — that section is the authoritative list; this stage does not maintain its own copy of it.**
  (trigger: an intent document is under review; outcome: the review's dimension set can never silently drift from AGENTS.md's own list)
- **Each dimension is reported as exactly one of `covered`, `gap`, or `not-applicable`: `covered` cites where in the intent it is addressed; `gap` names specifically what is missing; `not-applicable` states the reason that dimension does not apply to this objective.**
  (trigger: the review reaches a dimension; outcome: every dimension has an unambiguous, checkable disposition — never a vague "mostly fine")
- **This stage never rewrites, edits, or completes the intent document under review, and never invents content to make a gap look covered.**
  (trigger: a dimension is found to be a gap; outcome: the gap is reported honestly rather than papered over — the same "log gaps rather than fill them" discipline `record-decisions` codifies for a different artifact)
- **This review's output is a report, not a gate: nothing here blocks, delays, or is required before any dispatch — `sgt run` accepts an intent regardless of what this workflow reports about it (owner ruling, #201).**
  (trigger: the review completes; outcome: a Captain reads the report and decides for themselves what to do with it; no downstream mechanism treats a `gap` finding as a refusal)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether a given passage of the intent actually addresses a
  dimension (a citation) versus merely mentions related words.
- Phrasing each gap's "what's missing" and each not-applicable's reason.

### J1 — local choices allowed
- Formatting/ordering of the per-dimension report.

### J0 — must become `needs_input`
- The named intent document cannot be found or read.
- The document under review is not itself an intent (e.g. it is
  unrelated prose) and no reviewable text can be identified.

### Completion boundary
This stage may complete only when all eight dimensions have been
reported, each as exactly one of `covered`/`gap`/`not-applicable`, and
the intent document under review is byte-for-byte unchanged from what
this stage started with.

### Decision evidence
The per-dimension report is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
