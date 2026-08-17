# 30-independent-review: independent review

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-implement/output/README.md | L4 | upstream artifact produced by `20-implement` |

## Purpose

Every axis named in the brief's authoritative list runs as a separate, non-contaminating parallel review; outputs unblended.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

Every axis named in the brief's authoritative list runs as a separate, non-contaminating parallel review; outputs unblended.

## Behavior contract

- **Independent review before completion must run every axis named in the brief's authoritative axis list as separate parallel subagents whose contexts cannot contaminate each other, even if the loaded review skill itself names fewer axes, and their outputs must stay in separate, unblended, unreranked sections.**
  (trigger: a worker reaches its pre-completion independent-review gate; outcome: review coverage is deterministic and driven by the dispatching context (the brief), not silently narrowed by whatever generic skill text happens to be loaded)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- None beyond ordinary tool mechanics of dispatching the parallel sub-reviews.

### J1 — local choices allowed
- Formatting/ordering of the unblended per-axis output sections.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when every axis named in the brief's authoritative list has run as a separate, non-contaminating parallel review, with outputs unblended — never narrowed to whatever fewer axes a loaded review skill happens to name (J5).

### Decision evidence
The per-axis review outputs are this stage's own decision record.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
