# ADR 0016: A gate Work binds to the branch it reviews by sequenced takeover, not a fresh branch or a direct binding

**Status:** Accepted, 2026-08-17 (extracted; the underlying ruling and implementation land earlier, per FOUNDATION-1/§8.6 below).

## Context

ADR 0005 rules that gating becomes a dispatched Work rather than a Captain-
run pipeline, on the premise that Sergeant's own surfaces (isolated
worktrees, one Work owning its own journal-backed execution) already give
gating the isolation no-mistakes' branch-ownership model was standing in
for. That ruling left one mechanical question open: a gate Work needs to
review the *target* Work's branch, but a Sergeant surface mints its own
`sergeant/<work-id>` branch by default, and git's own worktree exclusivity
refuses two worktrees bound to the same branch while both exist.

`reference/proposal-foundation-rationalization.md` §8.6 ("How a gate Work
binds to the branch it reviews") investigated this directly — not merely
reasoned about it, but reproduced git's exclusivity refusal against scratch
repositories, recorded in `docs/gauntlet/runs/foundation-1/
8.6-gate-branch-binding.md` — and recommended what it calls Mechanism A,
sequenced branch takeover. That recommendation's binding-mechanism half is
now implemented in this repository, not merely proposed: `crate::runtime::
engine::branch_takeover_precondition` and `crate::runtime::surface::attach`
exist and are exercised by tests in `src/runtime/engine.rs` (search
`§8.6 Mechanism A`). The code itself cites "§8.6" as the sole record of why
it exists; once `reference/proposal-foundation-rationalization.md` moves to
the workspace repo under the product/workspace split (ADR 0014), that
citation would point at a document no longer in this repo. This ADR is the
extraction that keeps the ruling itself in the product repo, per ADR 0014
decision 17 ("extract before moving, for every document... anything
binding present behavior is extracted to an ADR in the product repo
first").

## Decision

**A gate Work never mints its own branch to review a target Work's output;
it attaches to the target's existing branch once the target has gone
terminal and torn down clean.**

1. **Dispatch precondition, answered fail-closed from journaled state
   alone.** `branch_takeover_precondition` decides, from the journal, whether
   a target's branch is safe to take over. "Safe" means the target is
   terminal *in the absorbing sense* — `completed` or `canceled` — and
   specifically never `failed`, because `failed` is retryable and a
   takeover racing an in-flight retry is exactly the hazard this precondition
   exists to prevent. The target's surface must also have already torn down.
   Any violation is a named refusal, not a silent wait.
2. **The takeover reuses the existing branch, never mints a new one.**
   `surface::attach` performs the takeover by reusing the same
   `create_branch: false` git operation `rematerialize` already runs in a
   different context — this is not new git machinery, it is an existing
   operation applied to a new caller.
3. **Attached bindings are distinguishable from ordinary ones.**
   `RepositoryBinding::origin` records whether a binding was attached via
   takeover or cut ordinarily, so downstream code and operators can tell
   the two apart. Ordinary `materialize` is unchanged by this decision.
4. **This closes the binding-mechanism half of §8.6 only.** Three things
   §8.6 explicitly left open remain open and are not settled by this ADR:
   the submission shape for "this Work reviews `<target-work-id>`" (no
   field on `SubmitContext`/`StartPlan` carries this yet); the operational
   response to each precondition failure (block-and-wait for a
   not-yet-terminal target, block-and-escalate a retained-dirty target's
   teardown to Captain rather than silently re-driving the target's own
   retry path, and treating a residual git-level takeover race the same as
   an ordinary `materialize` failure — fail closed to `blocked`, never
   auto-retried); and whether review independence survives when the
   reviewer is another Work rather than an external pipeline (§8.2,
   unmeasured). These remain product decisions for a future owner ruling,
   not inventions of this ADR.

## Alternatives considered

- **Bind the gate Work directly to the target's branch while the target's
  worktree still exists.** Rejected: git's own worktree exclusivity refuses
  two worktrees on the same branch simultaneously — reproduced directly
  against scratch repositories, not merely reasoned about.
- **Review a copy of the branch instead of the branch itself.** Rejected:
  this breaks no-mistakes' recovery of auto-fix commits onto the shipped
  branch — the whole point of gating is that fixes land on the branch that
  actually ships, not a disposable copy of it.

## Consequences

- §5.1 of the foundation-rationalization proposal (gating as a dispatched
  Work, ADR 0005) may now be dispatched for the binding mechanism's sake —
  the mechanical blocker ADR 0005 left unresolved is gone. It still cannot
  be dispatched end-to-end until the three items in Decision 4 above are
  ruled on.
- A target Work that ends `failed` is never eligible for takeover by
  design — a gate Work can only ever review work that reached a clean,
  non-retryable terminal state.
- The provenance pointer this ADR replaces (`§8.6` inside
  `reference/proposal-foundation-rationalization.md`) can now safely move to
  the workspace repo along with the rest of that document's argument
  record; the binding ruling itself no longer depends on that document
  remaining in this repo. `docs/gauntlet/runs/foundation-1/
  8.6-gate-branch-binding.md`, the investigation record, stays in this
  repo's own gauntlet ledger.

## Open questions

- The submission shape for "this Work reviews `<target-work-id>`" — a
  sibling entry point to `Engine::plan` that derives its workspace from the
  target's own bindings, reachable through a dedicated CLI/API verb rather
  than a mode flag on ordinary `sgt work submit`, is the shape §8.6
  recommends but does not implement.
- The operational response to each precondition failure (block-and-wait,
  block-and-escalate, fail-closed on takeover race) is named above as the
  right shape but not yet ruled on by an owner.
- Whether adversarial review independence survives when the reviewer is a
  Work rather than an external pipeline (§8.2) is unmeasured.
