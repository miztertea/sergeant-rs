# 16-probe-harness-readiness

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a worker polls whether its pane can receive a nudge

**Outcome:** a dead or still-blank pane is never reported ready

**Statement (the operative rule):** The tui readiness probe requires the target tmux pane to be alive and to have rendered at least one non-whitespace glyph before the pane can be considered ready to receive input.

## What must become true here (durable outcome)

A dead or still-blank pane is never reported ready — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0319`: The tui readiness probe never reports a pane ready on the very first observation of drawn output; a later, second observation is required, because a TUI's first painted frame may still be installing its input handlers.
- `BU-0320`: The tui readiness probe honors a configurable wall-clock settle time (SGT_HARNESS_SETTLE_SECONDS) as an additional minimum on top of the two-consecutive-observation rule, defaulting to none.
- `BU-0322`: Dispatching a readiness check to a probe identifier that is declared but not implemented fails loudly, naming the harness and the unimplemented probe, rather than silently treating the harness as never ready.
- `BU-0352`: When the bounded readiness gate's timeout elapses, the worker publishes a durable readiness_failed record (once) naming the notification, nonce, harness, and seconds waited.
- `BU-0353`: The readiness-timeout failure message explicitly states that no acknowledgement, acceptance, delivery, or action lease was fabricated and that the notification is still pending, directing the operator to confirm the harness renders a prompt and resume via the worker response-delivery step.

