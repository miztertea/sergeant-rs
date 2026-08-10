# 00-parse-and-sanitize: parse and sanitize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Raw review output is parsed and sanitized before anything downstream consumes it.

Trigger (workflow-level): A review pass (worker-mission's `30-independent-review`, or code-review) has produced findings.

## What must become true here (durable outcome)

Raw review output is parsed and sanitized before anything downstream consumes it.

## Behavior contract

- **Independent review findings are routed to tracked work as a bounded, evidence-preserving procedure: parse and sanitize the reviewer's structured output, retain a sanitized copy before any external side effect, route each actionable finding to exactly one deduplicated task, and — only once every finding has reached tracked work — publish a blocking gate if any finding is severe enough to block, or clear it otherwise.**
  (trigger: a review pass (standards, spec, readiness, etc.) produces structured findings; outcome: every actionable finding either reaches exactly one tracked-work item or is explicitly refused with a stated reason; a severe-enough finding blocks the worker until it is addressed)
  — `BU-P6-082`, `reference/sergeant-upstream/bin/sgt-review-findings` (L2)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
