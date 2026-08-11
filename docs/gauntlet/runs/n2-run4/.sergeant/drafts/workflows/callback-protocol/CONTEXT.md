# callback-protocol — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a callback profile is installed or invoked
- **Outcome:** delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `retry-delivery`'s.

## How the stages relate

`callback-protocol` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain. The list below is the corpus's own defensible-default ordering (source/behavior_id order), not a claim that each stage's input is the previous stage's output.

1. `01-resolve-callback-executable` — only a locally-installed, ownership-and-permission-verified executable can ever run as a callback, never a path supplied through request/fleet data
2. `02-register-origin` — the ID is validated to be opaque and rejects anything shaped like a real platform identifier
3. `03-sync-and-produce-events` — re-running sync is idempotent — it never fabricates a new event generation for state it has already classified
4. `04-enqueue-event` — the source identity is validated, never stored in plaintext, and re-use is idempotent rather than creating a duplicate event
5. `05-invoke-consumer` — the consumer receives a minimized, argument-free, environment-scrubbed invocation surface
6. `06-process-acknowledgement` — the event's next state is determined by this closed set of outcomes, with every malformed/unexpected response defaulting to pending (never silently ack'd)
7. `07-retry-delivery` — delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

