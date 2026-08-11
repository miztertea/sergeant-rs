# 04-work-through-map-session

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../03-chart-the-map/output/outcome.md | L4 | upstream evidence produced by `chart-the-map` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a work-through-the-map session begins with a map

**Outcome:** ticket selection is cheap and deterministic, and claimed before any work starts

**Statement (the operative rule):** A work-through-the-map session loads the low-resolution map view (not every ticket body); the ticket to work is the one the user named, or otherwise the first frontier ticket in order, and it is claimed by self-assignment before any work begins.

## What must become true here (durable outcome)

Ticket selection is cheap and deterministic, and claimed before any work starts — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1009`: A ticket's answer is not part of its body — it is recorded on resolution — and assets created while resolving are linked from the issue, not pasted into it.
- `BU-1011`: A research ticket is resolved by a /research subagent, used when knowledge from outside the current working directory is required.
- `BU-1012`: A prototype ticket is resolved by making a cheap, rough, concrete artifact via the /prototype skill to raise the discussion's fidelity, linked as an asset — used when 'how should it look/behave' is the key question.
- `BU-1013`: A grilling ticket is resolved via the /grilling and /domain-modeling skills, one question at a time — the default ticket type.
- `BU-1014`: A task ticket (manual work blocking a decision but not itself a decision) has the agent drive the work alone where it can (AFK) or hand the human a precise checklist (HITL); it is resolved once the work is done, and its answer records what was done plus any resulting facts later tickets depend on.
- `BU-1016`: A not-yet-specified item becomes a ticket once the question can be stated precisely (even if it's blocked and can't be acted on yet); it stays fog if it can't yet be phrased that sharply — fog is not pre-sliced into ticket-sized pieces.
- `BU-1017`: The Not yet specified section excludes anything already decided (Decisions so far), anything already a live ticket, and anything out of scope.
- `BU-1018`: Work identified as beyond the map's destination is recorded as out of scope, not as fog, and does not belong in Not yet specified, because the destination fixes the scope.
- `BU-1019`: Out-of-scope work never graduates; it returns only if the destination itself is redrawn, and then as a fresh effort, not a resumption of the old one.
- `BU-1020`: If a ticket that already exists turns out to sit past the destination, it is closed (unambiguously off the frontier) and one line is added to the Out of scope section (the gist, why it's out of scope, and a link to the closed ticket); it stays out of Decisions so far, since that section records only the route actually walked.
- `BU-1021`: Never more than one ticket is resolved per work-through-the-map session, except research tickets.
- `BU-1029`: A claimed ticket is resolved by zooming in as needed — fetching the full body of any related or closed ticket on demand, invoking the skills named in the map's Notes block, and defaulting to /grilling and /domain-modeling when in doubt.
- `BU-1030`: Once a ticket is resolved, the answer is posted as a resolution comment, the issue is closed, and a context pointer is appended to the map's Decisions so far.
- `BU-1031`: After a ticket's resolution is recorded, newly-surfaced tickets are added (create-then-wire); any now-specifiable fog is graduated and cleared from Not yet specified; a ticket found to sit beyond the destination is ruled out of scope rather than resolved on the route; and any other map parts the decision invalidates are updated or deleted.
- `BU-1032`: When the user runs unblocked tickets in parallel, other sessions may be editing the tracker concurrently, and this is an expected condition rather than an error.

