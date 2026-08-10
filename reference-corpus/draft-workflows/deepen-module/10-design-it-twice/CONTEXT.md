# 10-design-it-twice: design it twice

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-classify-dependencies/output/README.md | L4 | upstream artifact produced by `00-classify-dependencies` |

## Purpose

At least 3 independently generated, structurally different designs, each under a distinct constraint, compared on depth/locality/seam placement, ending in an opinionated recommendation.

Trigger (workflow-level): A module's interface needs redesign, or a port/adapter decision needs to be made deliberately rather than by default.

## What must become true here (durable outcome)

At least 3 independently generated, structurally different designs, each under a distinct constraint, compared on depth/locality/seam placement, ending in an opinionated recommendation.

## Behavior contract

- **When exploring alternative interfaces for a chosen deepening candidate, run a parallel-sub-agent pattern that produces several radically different designs before picking one, on the premise that a single first idea is unlikely to be the best.**
  (trigger: a deepening candidate has been chosen and alternative interface shapes should be explored before committing; outcome: multiple independently-produced interface designs exist and are compared before one is chosen)
  — `BU-P4-022`, `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (L3)
- **Before spawning parallel design sub-agents, first produce and show the user a framing of the problem space (constraints, dependency category, an illustrative sketch), then proceed immediately to spawning sub-agents without waiting for a reply.**
  (trigger: framing has been produced for a design-it-twice pass; outcome: the user is informed of the problem framing concurrently with sub-agent work starting, rather than gating on their reply)
  — `BU-P4-023`, `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (Process step 1, L17)
- **Produce at least three independently-generated, radically different interface designs for the same deepening candidate, each under an explicit distinguishing design constraint (e.g. minimal interface, maximal flexibility, optimize the common case, ports-and-adapters).**
  (trigger: the problem space has been framed for a design-it-twice pass; outcome: three or more genuinely distinct candidate interface designs exist for comparison)
  — `BU-P4-024`, `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (Process step 2, L21)
- **Present the several generated interface designs to the user one at a time so each can be absorbed, then compare them explicitly by depth (leverage at the interface), locality (where change concentrates), and seam placement, ending with an opinionated recommendation (including a hybrid if warranted) rather than a menu.**
  (trigger: multiple candidate designs have been produced; outcome: the user receives a structured comparison and a concrete recommendation, not a raw dump of options)
  — `BU-P4-026`, `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (Process step 3, L42)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
