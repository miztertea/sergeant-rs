# ADR 0005: Gating becomes a dispatched Work

**Status:** Accepted, 2026-08-14.

## Context

`docs/DEVELOPMENT.md`'s session-conduct rules currently state: "A workflow
stage or actor executing inside a worktree never invokes
`scripts/gate.sh`/no-mistakes itself — only the top-level orchestrating
session owns a shipping-gate run, matching the single-owner posture the
engine itself enforces on the data dir" (`BU-0041`, `BU-0122`, `BU-1196`).
In practice this meant every gate ran Captain-serial: the orchestrating
session drove `scripts/gate.sh`, then hand-plumbed `no-mistakes axi respond`
on findings and `no-mistakes axi sync --recover` to take custody of
pipeline commits, for every gate on every branch. This decision came out of
a live grilling interview held the same day as the cross-platform sprint
this rule was written for (`docs/gauntlet/runs/cross-platform-2026-08-14/plan.md`
lists "the gate-procedure reconciliation" as one of the items owed an owner
ruling before that sprint's later waves could run), prompted by that
sprint's own experience of the rule: three gates ran serially through
Captain on 2026-08-14, and one gate run was lost outright to a harness
kill mid-run.

## Decision

**The rule dissolves rather than being amended (D1).** The owner's ruling:
the existing "actor never invokes the gate" text is not a principle, it is
a workaround for how no-mistakes takes ownership of the whole branch while
it runs. What this repo actually values about no-mistakes is its process
stages — review, test, lint, docs — and the ICM workflow `validate-and-ship`
already adapts those stages to how sergeant-rs works. So the fix is not to
carve out an exception to the single-owner rule for gate runs; it is to
recognize that if the gate itself *is* the dispatched Work, and that Work
owns its own surface the way every other Work does, there is exactly one
owner and nothing left for the old rule to prevent. Sergeant's surfaces
(isolated worktrees, one Work one journal-backed execution) already provide
the isolation that no-mistakes' branch-ownership model was standing in for.
Gating becomes a dispatched Work: Captain adjudicates findings; Sgt
executes.

**No-mistakes stays inside the gate Work for now, and is rebuilt only where
matched (part of D1).** The counter-argument was raised and is recorded,
not smoothed over: no-mistakes' review agent is a known-good cold reader
with a real track record — across three Works' shipping gates on
2026-08-14, it found four real defects that the orchestrating session's own
review had already passed. A rebuilt ICM review stage would be a new brief
with no comparable track record, and "our own gate" could quietly become a
worse gate than the one being replaced. The decision is therefore to keep
no-mistakes' stages running inside the gate Work as-is at first, and only
rebuild a stage once there is evidence the replacement actually matches
what it replaces — not on the strength of the ICM workflow's existence
alone.

**The auto-fix / no-op / ask-user authority split is unchanged (part of
D1).** `validate-and-ship`'s `40-drive-gates` stage already classifies every
finding into one of three actions: `auto-fix` (mechanical/low-risk, the
actor may authorize on its own judgment), `no-op` (informational, nothing
to do), or `ask-user` (challenges the user's deliberate intent or touches
product behavior — a decision only the user can make, relayed verbatim and
never resolved autonomously). This decision does not touch that split; it
only changes who runs the pipeline that produces the findings, from Captain
by hand to a dispatched Work.

## Alternatives considered

**Amend the existing rule with an exception for gate Works** was the
shape the old text implicitly invited — keep "an actor never invokes the
gate" as the general rule and carve out gate-running Works as a special
case. Rejected in favor of dissolving the rule outright: an exception only
makes sense if there is still something for the general rule to protect
against once the gate is itself a properly surfaced, isolated Work, and the
interview's finding was that there is not — the single-owner posture the
old rule was protecting is already what every Work gets by construction.

**Rebuilding no-mistakes' stages as native ICM review immediately**, rather
than keeping no-mistakes inside the gate Work as a first cut, was
considered and rejected for the reason recorded above under D1: no-mistakes'
review agent has a measured track record from the same day this decision
was made (four real defects across three gates), and a rebuilt review stage
would start with none.

## Consequences

**Partially implemented.** The branch-binding mechanism this ADR's own Open
Questions deferred to §8.6 is now resolved and shipped: a sergeant surface
mints its own `sergeant/<work-id>` branch and cannot bind to a branch it did
not mint (confirmed by gauntlet FOUNDATION-1's `inv-one-owner-relocated`,
error severity), so a gate Work reviewing another Work's real branch needs a
mechanism that does not exist merely by gating becoming a Work.
`docs/gauntlet/runs/foundation-1/8.6-gate-branch-binding.md` investigated the
alternatives and recommended Mechanism A (sequenced branch takeover): dispatch
the gate Work only once the target Work is terminal and its surface has torn
down clean, then materialize the gate Work's surface by *attaching* to the
target's existing branch instead of minting a new one. That investigation's
items 1 and 2 now ship: `crate::runtime::engine::branch_takeover_precondition`
reads journaled state to answer, fail-closed and with a named reason, whether
a target Work's branch is safe to take over — terminal (absorbing: completed
or canceled; `failed` is deliberately excluded, since it is retryable and a
retry would rematerialize the target's own worktree back onto the branch a
gate surface just attached to) and its surface's teardown reported every
binding `Removed`. `crate::runtime::surface::attach` performs the takeover
itself, checking a gate Work's worktree out onto the target's real branch via
the same `create_branch: false` git-level operation `rematerialize` already
uses in a different context, with its own `RepositoryBinding::origin` field
(`BindingOrigin::Cut` vs `Attached { target_work_id }`) recording which shape
produced a binding — additive, so every binding journaled before this field
existed still deserializes as the `Cut` it always was. `materialize`'s own
minting path for an ordinary Work is untouched by any of this.

Investigation items 3 and 4 — a submission shape for "this Work reviews
`<target-work-id>`", and the fail-closed behavior for each of the three named
precondition-failure cases (target not terminal, target terminal but
surface retained-dirty, target's branch attached elsewhere for an unrelated
reason) — remain open, deliberately: they are product decisions (what
Captain's dispatch call looks like; whether a retained-dirty target blocks
for a human or routes through the target's own retry path) that a branch-
binding mechanism does not, by itself, answer. `branch_takeover_precondition`
already refuses each of those three cases with a distinctly-named
`BranchTakeoverError` and a stated reason (`TargetNotTerminal`,
`SurfaceNotClean`, and — for the git-level race the journal cannot see —
`surface::attach`'s own `SurfaceError`), so the *fail-closed answer* to "may
this proceed" exists; what is still undecided is who acts on a refusal and
how, which is exactly what items 3 and 4 were always scoped to settle, not a
gap this shipment introduced.

Gating stops being Captain-serial: each gate previously blocked the
orchestrator for roughly eight minutes, and durability was not free —
one of the three gates run on 2026-08-14 was lost entirely to a harness
kill mid-run. As a dispatched Work, a gate run becomes durable and
resumable through the same journal-and-recovery machinery every other Work
gets, rather than living only in an orchestrating session's own process
lifetime.

Captain's job changes shape: instead of driving `axi respond` and
`axi sync --recover` plumbing by hand for every gate, Captain's remaining
job is reading a gate Work's findings and deciding — the judgment step, not
the mechanical one.

The negative consequence recorded alongside this decision is real: keeping
no-mistakes embedded inside the gate Work, rather than rebuilding its
stages against this repo's own ICM review from day one, means this repo is
still leaning on an external pipeline's ownership model for the one stage
(review) that most needs a good cold reader. That dependency does not go
away with this decision; it is deliberately kept until there is evidence
sufficient to replace it stage by stage.

## Open questions

What specifically counts as evidence that a rebuilt stage has "matched"
the no-mistakes stage it would replace — a defect-count threshold, a
side-by-side run, an owner sign-off — was not specified in the interview.
Until that bar is named, "rebuild only where we can show we have matched
them" has no operational trigger and risks never firing, or firing on an
ad hoc judgment call each time.

The mechanics of how a gate Work is specified — what workflow stage
invokes `scripts/gate.sh`/no-mistakes, what its own findings-to-Work
schema looks like, whether it is a new named ICM workflow or a stage
folded into an existing one — are not decided here; this ADR records that
gating becomes a dispatched Work, not the shape of that Work.

**Narrowed.** One piece of that mechanics question is no longer open: how a
gate Work's surface reaches the branch it needs to review is answered and
implemented (§8.6, Consequences above). What remains open is everything
about *specifying* the gate Work itself — a submission shape for "this Work
reviews `<target-work-id>`" (no field on `SubmitContext`/`StartPlan` carries
that today), and the operational response to each of
`branch_takeover_precondition`'s fail-closed refusals (block and wait for the
target to go terminal; block and escalate a retained-dirty target to Captain
rather than auto-retrying the target's own run; treat a git-level takeover
race the same as an ordinary `materialize` failure). Both are product
decisions this investigation's own sizing (§8.6, "items 3 and 4") declined to
invent, and still should not be invented without an owner ruling — see
`reference/proposal-foundation-rationalization.md` §8.6 for the standing
recommendation.
