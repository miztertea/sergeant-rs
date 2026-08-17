# To Tickets
Draft workflow package — candidate **W32** `to-tickets` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Break a plan, spec, investigation, findings register, PR, or conversation into dependency-aware tracer-bullet work.

## Trigger

The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-load-project-context` | actor-stage (§6.4, judgment) | Project context is loaded. |
| `10-extract-decisions-and-unknowns` | actor-stage (§6.4, judgment) | An investigation ticket is created only for a genuinely blocking unknown, naming the exact artifact it must produce. |
| `20-confirm-breakdown` | actor-stage (§6.4, judgment) | Granularity, ownership and blocking edges are confirmed unless immediate publication was requested; new tickets stay open, cross-repo blockers recorded as counterpart ids plus merge order. |
| `40-report-frontier` | actor-stage (§6.4, judgment) | One worker per owning repo is the default; reporting is not authorization to dispatch. |

## Relationships to other workflows

- `00-load-project-context` delegates to **estate-navigation** (retargeted
  ICM-R3, 2026-08-16: `load-project` retired ABSORBED — its protective and
  context-resolution intents are already owned by `estate-navigation` and
  `sgt` itself, at a stronger rung; see `skills/estate-navigation/SKILL.md`).

## Authority envelope

This workflow receives an already-given artifact (a plan, spec,
investigation, findings register, PR, or conversation) to break down; it
does not itself decide whether that artifact should become Work.

### Workflow may decide
- Whether an unknown is genuinely blocking versus answerable from existing
  evidence (`10-extract-decisions-and-unknowns`).
- How to present the breakdown for confirmation, and what counts as "the
  project explicitly supports more" concurrency (`20-confirm-breakdown`,
  `40-report-frontier`).

### Workflow may not decide
- Automatically add td instructions to a repository's own guidance files
  as a side effect (`00-load-project-context`).
- Mark newly published tasks `in_progress`, or invent a native
  cross-repository dependency edge td cannot enforce
  (`20-confirm-breakdown`'s publish helper).
- Dispatch any ticket unless the user asked to begin implementation —
  reporting the frontier is never itself authorization
  (`40-report-frontier`).

### Human or Captain gates
- Unless immediate publication was requested, confirming the proposed
  breakdown's granularity, ownership, and blocking edges
  (`20-confirm-breakdown`).
- The evidence for whether an unknown is genuinely blocking is itself
  ambiguous or contested (`10-extract-decisions-and-unknowns`).
- A candidate ticket cannot be cleanly assigned a single owning repository
  (`20-confirm-breakdown`).
- A publish operation partially fails, leaving an internally-inconsistent
  dependency graph (`20-confirm-breakdown`'s publish helper).

### Decision record
Material decisions (loaded context, extracted unknowns, the confirmed
breakdown, published tickets, the reported frontier) are recorded in each
stage's own turn and surfaced through `needs_input` where applicable; this
workflow declares no separate decision-log file.

## Notes for reviewers

**N1 adjudication A4:** the former `30-publish` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `20-confirm-breakdown` as a helper invocation. `40-report-frontier`'s upstream Inputs pointer moves to `20-confirm-breakdown`. No renumbering: `00`, `10`, `20`, `40` remain correctly ordered without `30`. See `provenance.md`'s "Adjudication A4" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
