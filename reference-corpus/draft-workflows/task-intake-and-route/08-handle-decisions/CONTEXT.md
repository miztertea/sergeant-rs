# 08-handle-decisions: handle decisions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../07-monitor/output/README.md | L4 | upstream artifact produced by `07-monitor` |

## Purpose

Each gate resolved with a recorded human decision where required.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Each gate resolved with a recorded human decision where required.

## Behavior contract

- **Handle decisions: for needs_input, blocked, or ask-user gates, read the exact finding, obtain only genuinely missing user decisions, record them in td, and continue approved remediation without asking again merely to dispatch.**
  (trigger: a decision gate is hit; outcome: the gate is resolved with a recorded decision, not re-asked repeatedly)
  — `BU-P1-033`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L145, step 8)
- **Use sgt-respond, sgt-wake, or supported recovery only after reconciling status, response generation, pane identity, and handoff evidence.**
  (trigger: a resume action is about to be taken; outcome: resumption never proceeds on stale or unverified evidence)
  — `BU-P1-038`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L148, resume preconditions)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P1-038` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
