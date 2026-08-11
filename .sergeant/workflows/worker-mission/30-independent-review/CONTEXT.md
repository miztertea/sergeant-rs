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
  — `BU-P7-013`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 7. Independent {{REVIEW_AXIS_LABEL}}-axis review')

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
