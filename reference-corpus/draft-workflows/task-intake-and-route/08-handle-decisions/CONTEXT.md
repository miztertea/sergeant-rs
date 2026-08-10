# 08-handle-decisions: handle decisions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../06-execute/output/README.md | L4 | upstream artifact produced by `06-execute` |

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

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helpers (folded per N1 adjudication A4)

`07-monitor` carried no argument beyond the §6.5 deterministic-machinery boilerplate — no "Additional note" checkpoint argument — so it demotes by default and folds into this stage as a helper invocation performed before deciding how to resolve a gate:

- **Monitor real progress.** Require recent meaningful events or an active child operation plus exact process identity — parent-process liveness alone is insufficient; in OpenCode use a managed background watch and verify it started, falling back to bounded one-shot status checks rather than a blocking watch call when unavailable.
  — `BU-P1-032`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L144, step 7)

Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4): `BU-P1-032` and `BU-P1-038` above carry a durable identity/liveness/ownership policy that survives the pane, expressed in this project as the durable execution/session identity already journaled — not tmux. Per A11, this replaces the per-stage "read pane as..." reader-note block; the workflow-level `CONTEXT.md`'s "Notes for reviewers" section is the single, non-duplicated statement of this reading rule.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
