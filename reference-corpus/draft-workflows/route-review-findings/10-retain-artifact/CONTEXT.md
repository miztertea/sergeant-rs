# 10-retain-artifact: retain artifact

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-parse-and-sanitize/output/README.md | L4 | upstream artifact produced by `00-parse-and-sanitize` |

## Purpose

A sanitized copy is written to durable storage before any external side effect; the failure diagnostic names the retryable next action.

Trigger (workflow-level): A review pass (worker-mission's `30-independent-review`, or code-review) has produced findings.

## What must become true here (durable outcome)

A sanitized copy is written to durable storage before any external side effect; the failure diagnostic names the retryable next action.

## Behavior contract

- **A retained, sanitized artifact of parsed findings is written to durable storage before any external side effect (td calls), so a routing failure that happens after parsing never destroys the only copy of a review's findings; the artifact's location is included in the failure diagnostic as an explicit, retryable next action.**
  (trigger: findings have been parsed and validated, before they are routed to td; outcome: a review's evidence is never lost to a downstream infrastructural failure; it is always retryable from a durable, redacted copy)
  — `BU-P6-084`, `reference/sergeant-upstream/bin/sgt-review-findings` (L427-430)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
