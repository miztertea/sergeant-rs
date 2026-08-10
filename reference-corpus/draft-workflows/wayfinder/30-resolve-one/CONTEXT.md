# 30-resolve-one: resolve one

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-create-tickets/output/README.md | L4 | upstream artifact produced by `20-create-tickets` |

## Purpose

Claim, resolve by type, record the answer as a resolution and a one-line pointer; at most one non-research ticket per session.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

Claim, resolve by type, record the answer as a resolution and a one-line pointer; at most one non-research ticket per session.

## Behavior contract

- **When working through an existing map, load only the low-resolution map body (not every ticket's full content), choose the user-named ticket or else the first frontier ticket in order, and claim it by self-assignment before starting any work.**
  (trigger: a session begins working an existing wayfinder map; outcome: exactly one ticket is chosen and claimed before work starts, minimizing map-wide context loaded)
  — `BU-P4-098`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Work through the map, L122-123)
- **Every wayfinder ticket is either HITL (resolved through a live exchange with a human who speaks for themselves) or AFK (resolved by the agent alone); on a HITL ticket the agent must never answer on the human's behalf.**
  (trigger: a ticket is about to be resolved; outcome: human-authority decisions are never silently answered by the agent)
  — `BU-P4-085`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Ticket Types, L75)
- **A research-type wayfinder ticket surfaces a fact a decision is waiting on by reading documentation, third-party APIs, or local knowledge bases, and is resolved by delegating to a research subagent.**
  (trigger: a research-type ticket is chosen for resolution; outcome: the fact-finding work is delegated to a dedicated research procedure rather than performed ad hoc)
  — `BU-P4-086`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Ticket Types, L77)
- **A task-type wayfinder ticket is manual work that must happen before a decision can be made (e.g. provisioning access, moving data so its shape can be seen); it is the one ticket type that does rather than decides, and it earns its place only by unblocking a decision, not by delivering the destination itself.**
  (trigger: an unblocking prerequisite exists that is not itself a decision; outcome: the prerequisite is tracked and resolved as a task, without expanding the map's scope to include delivery work)
  — `BU-P4-087`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Ticket Types, L80)
- **Never resolve more than one wayfinder ticket per session, except that research tickets may be resolved in bulk (fired in parallel as subagents).**
  (trigger: a session is working through the map; outcome: each non-research decision gets a session's full, focused attention)
  — `BU-P4-093`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation, L105)
- **Recording a wayfinder ticket's resolution means posting the answer as a resolution comment, closing the issue, and appending a one-line context pointer to the map's Decisions-so-far section.**
  (trigger: a ticket's question has been answered; outcome: the map and the ticket both durably reflect the resolution in a fixed three-part sequence)
  — `BU-P4-099`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Work through the map, L125)
- **When an existing ticket turns out to sit past the destination, close it (making it unambiguously off the frontier) and record one line in the map's Out of scope section gisting why, rather than resolving it as if it were on the route.**
  (trigger: an existing ticket is found to be out of scope during resolution; outcome: the map accurately distinguishes decisions actually made from work ruled out of scope)
  — `BU-P4-092`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Out of scope, L101)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
