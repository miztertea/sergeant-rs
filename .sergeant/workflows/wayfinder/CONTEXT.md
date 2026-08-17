---
known_gap_refs:
  - ref: domain-modeling
    reason: cross-package materialization gap in the @@name shared-context catalog; only frozen upstream evidence exists, no .sergeant/common/contexts/domain-modeling.md yet
  - ref: ticket-shaping
    reason: same materialization gap; this package is the shared-by owner but has not yet materialized the file
  - ref: triage-state-machine
    reason: same materialization gap; this package is the shared-by owner but has not yet materialized the file
---

# Wayfinder
Draft workflow package — candidate **W33** `wayfinder` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Map an unfamiliar frontier of a codebase or problem space, ticket-izing decisions and resolving them one at a time.

## Trigger

A destination is named that requires mapping fog before it can be reached.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-name-destination` | actor-stage (§6.4, judgment) | The destination is named via a grilling/domain-modeling session; scope is settled first. |
| `10-map-frontier` | actor-stage (§6.4, judgment) | Breadth-first mapping; stop and do not create a map if no fog exists; specifiable decisions become child issues first, blocking edges wired in a second pass. |
| `30-resolve-one` | actor-stage (§6.4, judgment) | Claim, resolve by type, record the answer as a resolution and a one-line pointer; at most one non-research ticket per session. |
| `40-regraduate-fog` | actor-stage (§6.4, judgment) | Remaining fog is re-evaluated; the run loops back to `10-map-frontier` if fog remains. |

## Relationships to other workflows

- `00-name-destination` delegates to **grilling**.

## Authority envelope

This workflow receives an already-named (or, at `00-name-destination`,
about-to-be-settled-live) destination and a bounded outcome.

### Workflow may decide
- Whether frontier-mapping surfaces enough fog to justify a map at all, or
  whether the whole journey is small enough for one session (`10-map-frontier`).
- Which foreseen decisions are sharp enough to become tickets now versus
  staying in the fog (`10-map-frontier`).
- Which ticket to resolve next when the user has not named one, and how to
  classify a ticket's type (`30-resolve-one`).

### Workflow may not decide
- Answer on a human's behalf on a HITL ticket (`30-resolve-one`).
- Resolve more than one non-research ticket per session, or skip claiming a
  ticket by self-assignment before starting work.
- Silently graduate out-of-scope work into a ticket.

### Human or Captain gates
- No fog found during frontier-mapping: the stage stops and asks the user
  how to proceed rather than creating a trivial map (`10-map-frontier`).
- Any HITL ticket's live exchange with the human (`30-resolve-one`).

### Decision record
Material decisions are recorded on the map itself (Decisions-so-far,
Not yet specified, Out of scope) and in each ticket's resolution comment;
this workflow declares no separate decision-log file.

## Refer by name

Every map and ticket is referred to by its name (title) in all
human-facing narration — narration, the map's own Decisions-so-far — never
by a bare id, number, or slug. The id and URL ride inside the name, never
stand in for it.

This package is also the shared-by owner of `@@ticket-shaping` and
`@@triage-state-machine` (`sergeant-rs-workspace/knowledge/evidence/reference-corpus/shared-context-map.md` Part 2).
As with `@@domain-modeling` above, no `.sergeant/common/contexts/
{ticket-shaping,triage-state-machine}.md` file exists in this repo yet —
only frozen upstream evidence; this is a cross-package materialization gap
in the `@@name` shared-context catalog generally, not specific to this
package.

## Notes for reviewers

**N1 adjudication A4:** the former `20-create-tickets` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `10-map-frontier` as a helper invocation. `30-resolve-one`'s upstream Inputs pointer moves to `10-map-frontier`. No renumbering: `00`, `10`, `30`, `40` remain correctly ordered without `20`. See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/wayfinder.md`'s "Adjudication A4" section.

## Provenance

See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/wayfinder.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
