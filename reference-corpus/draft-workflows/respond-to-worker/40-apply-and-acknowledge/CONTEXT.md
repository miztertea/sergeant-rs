# 40-apply-and-acknowledge: apply and acknowledge

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-deliver-and-accept/output/README.md | L4 | upstream artifact produced by `30-deliver-and-accept` |

## Purpose

Decision applied once, truthful status restored, applied id/generation/status recorded, then acknowledged from the owning context.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

Decision applied once, truthful status restored, applied id/generation/status recorded, then acknowledged from the owning context.

## Behavior contract

- **A response can only be acknowledged when it is the exact pending response — matching response ID and a well-formed positive gate generation number — so an acknowledgement can never accidentally consume a different, superseding response.**
  (trigger: a worker acknowledges a specific response by ID; outcome: an acknowledgement is bound to one exact response identity and generation, never a wildcard match)
  — `BU-P6-032`, `reference/sergeant-upstream/bin/sgt-ack-response` (L45-49)
- **An acknowledged response's terminal outcome must be internally consistent: a status of done requires a non-empty result already present, and a status of failed requires a non-blank reason string, or the acknowledgement is refused.**
  (trigger: a response is being acknowledged against a terminal worker status; outcome: a terminal status is never accepted as evidence without the substance (result or reason) that makes it a real terminal outcome)
  — `BU-P6-034`, `reference/sergeant-upstream/bin/sgt-ack-response` (L88-94)
- **Acknowledging a response must verify the caller-provided response ID matches the pending response, the requesting pane's identity matches the recorded worker pane, and the worker's post-application status/proof file is present and valid — each check refusing (and leaving the pending response untouched) before any archive or acknowledgement state is published.**
  (trigger: sgt-ack-response is invoked to consume a delivered response; outcome: acknowledgement cannot be forged by the wrong pane, the wrong response ID, or a fabricated proof; every validation failure leaves the original response fully intact for a correct retry)
  — `BU-P7-041`, `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 37-59)
- **An archived acknowledgement record with an empty (unset) applied-status field must not be treated as matching a proof file with no status= line — this specific empty-vs-empty comparison used to be silently accepted as an already-converged replay, which is a false 'already delivered' the finalizer must refuse.**
  (trigger: sgt-ack-response replays against a pre-existing archive entry; outcome: convergence/replay logic never accepts a degenerate empty-equals-empty comparison as proof of a genuinely completed acknowledgement)
  — `BU-P7-044`, `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 321-347)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P7-041` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
