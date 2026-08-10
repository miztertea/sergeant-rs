# 00-publish-or-clear-gate: publish or clear gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

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

## Helpers (folded per N1 adjudication A4)

This workflow originally decomposed the routing procedure into four stages (`00-parse-and-sanitize`, `10-retain-artifact`, `20-route-each`, `30-publish-or-clear-gate`). Per N1 adjudication A4 (finding N1-BH-02), the first three carried no argument beyond the §6.5 deterministic-machinery boilerplate — none offered an "Additional note" checkpoint argument — so all three demote by default and fold into this stage (renamed from `30-publish-or-clear-gate`, now the workflow's sole stage) as helper invocations the actor performs before exercising the gate judgment:

- **Parse and sanitize.** Raw review output is parsed and sanitized before anything downstream consumes it.
  — `BU-P6-082`, `reference/sergeant-upstream/bin/sgt-review-findings` (L2)
- **Retain artifact.** A sanitized copy is written to durable storage before any external side effect (td calls), so a routing failure that happens after parsing never destroys the only copy of a review's findings; the artifact's location is included in the failure diagnostic as an explicit, retryable next action.
  — `BU-P6-084`, `reference/sergeant-upstream/bin/sgt-review-findings` (L427-430)
- **Route each.** Each finding is deduplicated against an existing tracked-work item using a marker scoped to the exact review axis, source, finding ID, parent mission, and branch — never axis/source/id alone. A matched existing tracked-work item is only ever updated when its stored content digest still matches the incoming finding's digest; a divergent stored body is refused and left completely untouched. Deduplication is scoped to the parent task and branch, not applied globally. The router refuses findings whose review-artifact composition or comparison the owner explicitly ruled must be rejected.
  — `BU-P6-085`, `BU-P6-086`, `BU-P7-063`, `BU-P7-064`, `reference/sergeant-upstream/bin/sgt-review-findings` (L524-528, L586-592), `reference/sergeant-upstream/tests/sgt-review-findings-test.sh` (lines 492, 541)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
