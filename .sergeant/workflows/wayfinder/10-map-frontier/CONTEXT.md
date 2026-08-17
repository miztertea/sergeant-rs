# 10-map-frontier: map frontier

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-name-destination/output/README.md | L4 | upstream artifact produced by `00-name-destination` |

## Purpose

Breadth-first mapping; stop and do not create a map if no fog exists; specifiable decisions then become child issues first, with blocking edges wired in a second pass.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

Breadth-first mapping; stop and do not create a map if no fog exists; specifiable decisions become child issues first, blocking edges wired in a second pass.

## Behavior contract

- **If breadth-first frontier-mapping surfaces no fog at all -- the whole journey is small enough for one session -- stop chartering, do not create a map, and ask the user how they would like to proceed instead.**
  (trigger: frontier-mapping during charting surfaces no remaining fog; outcome: a wayfinder map is not created for work that doesn't actually need one)
- **The map is deliberately incomplete: only decisions sharp enough to phrase precisely become tickets now, and everything else that's foreseeable but not yet phraseable stays recorded loosely as fog rather than being pre-sliced into ticket-sized pieces.**
  (trigger: charting or updating a map; outcome: the map never overcommits to decisions that aren't yet specifiable)
- **Whether something belongs in a ticket or in the fog is decided by whether the question can already be stated precisely, not by whether it can already be answered.**
  (trigger: deciding whether a foreseen decision should become a ticket now or stay in the fog; outcome: ticket-vs-fog placement is decided by a consistent, explicit test)
- **Out-of-scope work never belongs in the fog section, because fog only gathers toward the destination; work beyond the destination is recorded in its own Out of scope section instead, and out-of-scope work never later graduates into a ticket unless the destination itself is redrawn as a fresh effort.**
  (trigger: work is identified that lies beyond the chartered destination; outcome: scope creep is recorded explicitly rather than silently absorbed into the fog or the ticket graph)

## Helper invocation: create tickets

Demoted from a standalone stage (`20-create-tickets`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed once the frontier is mapped.

**Rung-rationale correction (ICM-R3, 2026-08-16):** the prior text here claimed "no `kind = \"execute\"` stage exists in the current engine" as part of this fold's justification. That is false as of this branch: `.sergeant/workflows/repo-to-icm/workflow.toml`'s `65-self-check` is a live `kind = "execute"` stage. Whether this two-pass ticket-creation fold should instead ride on a `kind = "execute"` stage after this actor stage is a real open question raised at `research/00-investigate/CONTEXT.md`'s equivalent correction and parked there, not resolved here. Until that's decided, the acting harness performs the ticket-creation operation itself:

- **When creating a wayfinder map, create the tickets that can already be specified as child issues first, then wire their blocking edges in a second pass, because issues need ids before they can reference each other.**
  (trigger: specifiable decisions have been identified during charting; outcome: the resulting ticket set has correct blocking edges despite the two-pass creation order)

## Helper invocation: map and ticket structure

The structural mechanics the map and its tickets depend on, extracted at N1 and already classified as this package's own workflow-local helper content (`sergeant-rs-workspace/knowledge/evidence/reference-corpus/helper-map.md` "Workflow-local helpers") but never landed in this stage until now:

- **The map is a single issue on the repo's issue tracker, labelled `wayfinder:map`; its tickets are child issues of the map.**
- **The map is an index, not a store: it lists decisions made and points at the tickets that hold their detail, gisting and linking rather than restating.**
- **Each ticket is sized to one ~100K-token agent session and carries exactly one `wayfinder:<type>` label (`research`, `prototype`, `grilling`, `task`).**
- **A session claims a ticket by self-assignment before any work; the assignee is the claim, and an open, unassigned ticket is unclaimed.**
- **Blocking uses the tracker's native dependency relationship, falling back to a body convention only if the tracker lacks native blocking.**
- **Open tickets are not listed inline on the map; they are found by query.**

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Whether a foreseen decision is sharp enough to become a ticket now, or belongs in the fog.
- Which work is out of scope versus still-fog.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- Frontier-mapping surfaces no fog at all: stop, do not create a map, and ask the user how they would like to proceed.

### Completion boundary
This stage may complete only once the frontier has been mapped breadth-first, specifiable tickets are created and blocking edges wired, and remaining fog is recorded in Not yet specified — or the stage has stopped at the J0 no-fog case above.

### Decision evidence
The map itself (Not yet specified, Out of scope, the ticket set) is this stage's own durable record.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
