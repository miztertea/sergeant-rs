# 20-confirm-breakdown: confirm breakdown

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-extract-decisions-and-unknowns/output/README.md | L4 | upstream artifact produced by `10-extract-decisions-and-unknowns` |

## Purpose

Granularity, ownership and blocking edges are confirmed unless immediate publication was requested; new tickets are then published, staying open, with cross-repo blockers recorded as counterpart ids plus merge order.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

Granularity, ownership and blocking edges are confirmed unless immediate publication was requested; new tickets stay open, with cross-repo blockers recorded as counterpart ids plus merge order.

## Behavior contract

- **Unless the user explicitly asked to publish immediately, present the proposed ticket breakdown first and ask only whether granularity, ownership, and blocking edges are correct -- do not re-ask about decisions already made.**
  (trigger: a candidate ticket breakdown has been drafted; outcome: the user confirms structural correctness before tickets are published)

## Ticket quality rules

Landed ICM-R3, 2026-08-16: these nine rules were already dispositioned
`skill: to-tickets` by a later, separate pass (`docs/icm/
agents-invariant-dispositions.md` lines 197-205), which found they belong
to this stage's own granularity/ownership/readiness judgment, but that
pass's own scope excluded landing content in workflows — a promotion gap,
not a placement error.

- **Prefer vertical slices that produce independently verifiable behavior when drafting tickets.**
- **Keep each ticket small enough for one fresh agent context.**
- **Assign exactly one owning repository to each implementation ticket.**
- **Use expand-migrate-contract for mechanical changes that cannot remain green as a vertical slice.**
- **Create epics for coherent programs of work, not as substitutes for executable tickets.**
- **Never duplicate an existing task tracker task or GitHub issue.**
- **Preserve stable finding IDs such as `RBAC-P1-004` or `DATA-P0-002` in ticket titles.**
- **A ticket is not ready unless its acceptance criteria are observable and its blockers are accurate.**

## Bounded judgment

Apply `@@bounded-judgment`.

### J4 — explicit user or bound Work decision
- The user's own explicit "publish immediately" request governs whether to ask for confirmation at all.

### J2 — delegated to this stage
- How to present the breakdown for confirmation.
- Applying the ticket quality rules above to judge granularity, ownership, and readiness.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- A candidate ticket cannot be cleanly assigned a single owning repository (conflicts with an irreducibly cross-repo unit of work).
- A publish operation partially fails (e.g. an epic and some tickets are created, then a later dependency call fails), leaving an internally-inconsistent dependency graph: report the partial state and stop rather than guessing whether to roll back or continue.

### Completion boundary
This stage may complete only once the breakdown is confirmed (or immediate publication was explicitly requested) and tickets are published per the helper invocation below — or the stage has stopped at one of the J0 cases above.

### Decision evidence
The confirmed breakdown and published tickets are this stage's own durable output, recorded per `output/README.md`.

## Helper invocation: publish

Demoted from a standalone stage (`30-publish`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed once the breakdown is confirmed.

**Rung-rationale correction (ICM-R3, 2026-08-16):** the prior text here claimed "no `kind = \"execute\"` stage exists in the current engine" as part of this fold's justification. That is false as of this branch: `.sergeant/workflows/repo-to-icm/workflow.toml`'s `65-self-check` is a live `kind = "execute"` stage. Whether "publish to td, with in_progress and cross-repo-blocker discipline" should become a mechanical execute-stage check rather than trusted to this stage's own judgment is a real open question this raises but does not resolve — parked as a follow-on finding, not resolved here. Until that's decided, the acting harness performs the publish operation itself:

- **Do not mark newly published tasks in_progress; that transition belongs to dispatch or the worker that later starts the work. New tickets remain open until execution actually begins.**
  (trigger: tickets/epics are being published to td; outcome: ticket status accurately reflects that planning, not execution, has occurred)
- **td dependencies are repository-local; for cross-repository blockers, record the counterpart repo/ticket id and exact merge order in both descriptions/logs rather than inventing a native dependency edge td cannot enforce across separate databases.**
  (trigger: a ticket is blocked by a ticket in a different repository's td database; outcome: cross-repo blocking is tracked honestly as a documented convention, not a fabricated native edge)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
