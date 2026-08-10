# 30-publish-or-clear-gate: publish or clear gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-route-each/output/README.md | L4 | upstream artifact produced by `20-route-each` |

## Purpose

The gate is published or cleared only after every finding reached tracked work.

Trigger (workflow-level): A review pass (worker-mission's `30-independent-review`, or code-review) has produced findings.

## What must become true here (durable outcome)

The gate is published or cleared only after every finding reached tracked work.

## Behavior contract

- **Independent review findings are routed to tracked work as a bounded, evidence-preserving procedure: parse and sanitize the reviewer's structured output, retain a sanitized copy before any external side effect, route each actionable finding to exactly one deduplicated task, and — only once every finding has reached tracked work — publish a blocking gate if any finding is severe enough to block, or clear it otherwise.**
  (trigger: a review pass (standards, spec, readiness, etc.) produces structured findings; outcome: every actionable finding either reaches exactly one tracked-work item or is explicitly refused with a stated reason; a severe-enough finding blocks the worker until it is addressed)
  — `BU-P6-082`, `reference/sergeant-upstream/bin/sgt-review-findings` (L2)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
