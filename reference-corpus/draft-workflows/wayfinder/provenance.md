# Provenance — Wayfinder

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W33** `wayfinder`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-075` | Wayfinder plans a body of work too large for one agent session as a shared map of decision tickets on the issue tracker, resolving them one at a time until the way to a named destination is clear. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (frontmatter description, L3) |

## Stages

### `00-name-destination`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-094` | Charting a wayfinder map first names the destination via a grilling/domain-modeling session (settling scope first), then maps the frontier breadth-first across the whole space rather than deep on one thread. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L111) |

### `10-map-frontier`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-095` | If breadth-first frontier-mapping surfaces no fog at all -- the whole journey is small enough for one session -- stop chartering, do not create a map, and ask the user how they would like to proceed instead. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L112) |
| `BU-P4-088` | The map is deliberately incomplete: only decisions sharp enough to phrase precisely become tickets now, and everything else that's foreseeable but not yet phraseable stays recorded loosely as fog rather than being pre-sliced into ticket-sized pieces. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Fog of war, L82) |
| `BU-P4-089` | Whether something belongs in a ticket or in the fog is decided by whether the question can already be stated precisely, not by whether it can already be answered. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Fog of war, L88) |
| `BU-P4-091` | Out-of-scope work never belongs in the fog section, because fog only gathers toward the destination; work beyond the destination is recorded in its own Out of scope section instead, and out-of-scope work never later graduates into a ticket unless the destination itself is redrawn as a fresh effort. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Out of scope, L97) |

### `10-map-frontier` (helper invocation, folded from demoted `20-create-tickets`)

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-096` | When creating a wayfinder map, create the tickets that can already be specified as child issues first, then wire their blocking edges in a second pass, because issues need ids before they can reference each other. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L114) |

### `30-resolve-one`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-098` | When working through an existing map, load only the low-resolution map body (not every ticket's full content), choose the user-named ticket or else the first frontier ticket in order, and claim it by self-assignment before starting any work. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Work through the map, L122-123) |
| `BU-P4-085` | Every wayfinder ticket is either HITL (resolved through a live exchange with a human who speaks for themselves) or AFK (resolved by the agent alone); on a HITL ticket the agent must never answer on the human's behalf. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Ticket Types, L75) |
| `BU-P4-086` | A research-type wayfinder ticket surfaces a fact a decision is waiting on by reading documentation, third-party APIs, or local knowledge bases, and is resolved by delegating to a research subagent. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Ticket Types, L77) |
| `BU-P4-087` | A task-type wayfinder ticket is manual work that must happen before a decision can be made (e.g. provisioning access, moving data so its shape can be seen); it is the one ticket type that does rather than decides, and it earns its place only by unblocking a decision, not by delivering the destination itself. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Ticket Types, L80) |
| `BU-P4-093` | Never resolve more than one wayfinder ticket per session, except that research tickets may be resolved in bulk (fired in parallel as subagents). | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation, L105) |
| `BU-P4-099` | Recording a wayfinder ticket's resolution means posting the answer as a resolution comment, closing the issue, and appending a one-line context pointer to the map's Decisions-so-far section. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Work through the map, L125) |
| `BU-P4-092` | When an existing ticket turns out to sit past the destination, close it (making it unambiguously off the frontier) and record one line in the map's Out of scope section gisting why, rather than resolving it as if it were on the route. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Out of scope, L101) |

### `40-regraduate-fog`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-076` | Wayfinder defaults to planning only: each ticket resolves a decision and the map is done once nothing is left to decide, not once the underlying work is executed; an effort may explicitly override this default in its own Notes to carry execution into the map. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Plan don't do, L13) |
| `BU-P4-097` | After creating research-type tickets during charting, immediately fire a research subagent per ticket in parallel to resolve it, capturing findings on a throwaway branch with a context pointer back to the ticket. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L115) |
| `BU-P4-100` | Because unblocked tickets may be worked in parallel by other users, a session working through the map should expect other sessions to be editing the tracker concurrently. | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Work through the map, L128) |

## Adjudication A4

- **`20-create-tickets` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `10-map-frontier` as a helper invocation; `BU-P4-096` moves with it. The stage directory is removed; `30-resolve-one`'s Inputs table now points to `10-map-frontier/output/README.md`. No renumbering: `00`, `10`, `30`, `40` remain correctly ordered without `20`.
- **`40-regraduate-fog` — not a machinery stage, no A4 action.** It is classified `actor-stage (§6.4, judgment)` with its own "Judgment required" section; its "Additional note" (G7 engine-gap discussion) is a genuine actor-stage argument, not the §6.5 boilerplate, so it is outside A4's scope entirely.

