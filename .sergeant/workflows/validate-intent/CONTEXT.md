# Validate Intent
Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Review an intent document against AGENTS.md's Captain intent discipline
(the eight dimensions), reporting each dimension covered, gapped, or
not-applicable.

## Trigger

Captain wants an intent document checked for coverage before an
expensive or dangerous dispatch — optional tooling, invoked deliberately,
never a required gate.

## Stages

| Stage | Ladder rung | Durable outcome |
|---|---|---|
| `00-review-intent` | actor-stage (judgment) | Every one of the eight dimensions is reported `covered`, `gap` (naming what's missing), or `not-applicable` (with a reason); the intent itself is never rewritten and no gap is ever filled with invented content. |

## Authority envelope

This workflow receives an already-admitted Work whose intent names an
existing intent document to review (the same kind of text a Captain
composes, or is considering composing, for another Work's
`--intent-file`).

### Workflow may decide
- How to phrase a gap finding (what specifically is missing) and a
  not-applicable finding (why that dimension does not apply to this
  objective).

### Workflow may not decide
- Rewrite, edit, or complete the intent document under review.
- Invent content for a dimension the intent does not cover, to make it
  look covered.
- Treat this review's output as a pass/fail gate on any dispatch — it is
  a report, and nothing downstream is required to act on it.

### Human or Captain gates
- None beyond the stage's own `needs_input` triggers (see
  `00-review-intent/CONTEXT.md`).

### Decision record
The per-dimension report is this workflow's own durable output; this
single-stage workflow declares no separate decision-log file.

## Notes for reviewers

This package is optional Captain tooling (#201), never a mandatory
validator: `sgt` itself validates only `--intent-file`'s mechanics
(symlink, regular-file, size, UTF-8 — `sgt run --help`), never the eight
dimensions, and nothing in the engine or in any other workflow's
admission requires this package to have run. It exists so a Captain can
check an intent's own coverage before spending a worker on it, by choice.
