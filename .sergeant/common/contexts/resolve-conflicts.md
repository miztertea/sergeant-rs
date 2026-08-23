# Resolve conflicts

Resolved as `@@resolve-conflicts` from
`.sergeant/common/contexts/resolve-conflicts.md` per
`docs/icm/convention.md` §4. Shared stage context, two or more consumers:
`implement-change/10-implement`, `remediate-findings/30-implement-accepted`,
`validate-and-ship/50-reconcile-custody`. This is the retired
`resolving-merge-conflicts` package's entire content, carried forward at
the rung it earns as shared text rather than a standalone workflow.

Resolve an in-progress conflict — a git merge, a rebase, or a structured
branch-sync state — without inventing behavior and without aborting.

## Contract

- **Research both sides' intent before resolving any hunk**: commit
  messages, PRs, issues/tickets. Resolution follows traced intent, never
  a guess at what the conflicting change was probably trying to do.
- **Preserve both sides' intent where possible**; where genuinely
  incompatible, pick the side matching the run's own stated goal and
  record the trade-off.
- **Never invent new behavior** to paper over a hunk neither side's
  traced intent resolves.
- **Never abort.** The merge, rebase, or sync is always carried to
  completion — resolved, or escalated with the conflict left in place and
  the decision named.
- **After resolving, run the project's own automated checks** (typecheck,
  then tests, then format) and fix anything the merge broke before
  considering the conflict closed.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a hunk whose correct resolution is not
  derivable from either side's traced intent, or two sides that are
  genuinely irreconcilable with no discoverable stated goal to break the
  tie — record both intents, state the trade-off, and ask rather than
  resolving the tie unilaterally.
- **J2 the caller retains:** which primary sources to inspect when
  tracing intent, and which side matches the run's stated goal when a
  trade-off is required.
- **J1 the caller retains:** none beyond ordinary tool mechanics.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
