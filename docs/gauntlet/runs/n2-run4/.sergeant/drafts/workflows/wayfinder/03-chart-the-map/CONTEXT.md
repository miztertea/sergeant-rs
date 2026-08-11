# 03-chart-the-map

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../02-claim-ticket/output/outcome.md | L4 | upstream evidence produced by `claim-ticket` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a loose idea too big for one session is being charted

**Outcome:** every subsequent charting step is shaped by an already-fixed destination

**Statement (the operative rule):** Charting a map for a loose, oversized idea starts by naming the destination first (via /grilling and /domain-modeling), since naming it shapes every ticket and fixes the scope before anything else is planned.

## What must become true here (durable outcome)

Every subsequent charting step is shaped by an already-fixed destination — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1023`: The frontier is mapped breadth-first across the whole space rather than deep on one thread; if this surfaces no fog at all — the way to the destination is already clear — no map is created, and the actor stops to ask the user how they'd like to proceed instead.
- `BU-1024`: Once fog is surfaced, the map issue is created labelled `wayfinder:map` with Destination and Notes filled in, Decisions-so-far left empty, and the surfaced fog sketched into Not yet specified.
- `BU-1025`: Tickets that can be specified now are created as children of the map, then blocking edges between them are wired in a second pass (since issues need ids before they can reference each other); wiring sorts tickets into the frontier versus blocked, and anything still unspecifiable stays in the fog.
- `BU-1026`: A /research subagent is fired for each research ticket just created, in parallel, capturing its findings on a throwaway `research/<name>` branch with a context pointer from the ticket.
- `BU-1027`: Charting stops once complete — it is only one session's work, and it hand-resolves nothing.

