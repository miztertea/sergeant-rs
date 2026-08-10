# 10-map-frontier: map frontier

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-name-destination/output/README.md | L4 | upstream artifact produced by `00-name-destination` |

## Purpose

Breadth-first mapping; stop and do not create a map if no fog exists.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

Breadth-first mapping; stop and do not create a map if no fog exists.

## Behavior contract

- **If breadth-first frontier-mapping surfaces no fog at all -- the whole journey is small enough for one session -- stop chartering, do not create a map, and ask the user how they would like to proceed instead.**
  (trigger: frontier-mapping during charting surfaces no remaining fog; outcome: a wayfinder map is not created for work that doesn't actually need one)
  — `BU-P4-095`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L112)
- **The map is deliberately incomplete: only decisions sharp enough to phrase precisely become tickets now, and everything else that's foreseeable but not yet phraseable stays recorded loosely as fog rather than being pre-sliced into ticket-sized pieces.**
  (trigger: charting or updating a map; outcome: the map never overcommits to decisions that aren't yet specifiable)
  — `BU-P4-088`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Fog of war, L82)
- **Whether something belongs in a ticket or in the fog is decided by whether the question can already be stated precisely, not by whether it can already be answered.**
  (trigger: deciding whether a foreseen decision should become a ticket now or stay in the fog; outcome: ticket-vs-fog placement is decided by a consistent, explicit test)
  — `BU-P4-089`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Fog of war, L88)
- **Out-of-scope work never belongs in the fog section, because fog only gathers toward the destination; work beyond the destination is recorded in its own Out of scope section instead, and out-of-scope work never later graduates into a ticket unless the destination itself is redrawn as a fresh effort.**
  (trigger: work is identified that lies beyond the chartered destination; outcome: scope creep is recorded explicitly rather than silently absorbed into the fog or the ticket graph)
  — `BU-P4-091`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Out of scope, L97)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
