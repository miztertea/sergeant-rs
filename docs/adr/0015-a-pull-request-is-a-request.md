# ADR 0015: A pull request is a request — the merge is the authority boundary, not the artifact

**Status:** Accepted, 2026-08-17.

## Context

The SPLIT-1 grading unit (`docs/gauntlet/contracts/SPLIT-1.md`) raised an
error-severity finding against
`reference/proposal-product-workspace-split.md` §9: the proposal authorised
an unattended overnight run to draft and open a PR rewriting `AGENTS.md`,
labelled that authority "J2 — delegated drafting," and cited ADR 0014 as its
source. ADR 0014 rules on *what* `AGENTS.md` should contain. It says nothing
about who may execute the rewrite, unattended, with a morning review as the
only checkpoint. Captain wrote both the proposal and the section granting
Captain that authority.

The owner ruled directly:

> "I think that yes a pr is a request, which means it is not inherently
> approved. Therefore you may propose a full rewrite expressed as a pull
> request but you may not accept that pull request into truth (main)."

An independent adversarial refuter, working blind and instructed to default
to REFUTED, reached the same conclusion in the same hour on the same
grounds. Both arrivals are recorded in
`docs/gauntlet/runs/split-1/adjudication.md`.

The general question this settles is larger than that one overnight run:
`.sergeant/common/contexts/bounded-judgment.md` describes J2 and J0 in terms
that read as though *producing* something is the decision. Every dispatched
Work produces a branch. If producing a reviewable artifact required
pre-approval of its contents, no Work could do anything at all.

## Decision

**Producing a reviewable proposal is never the material decision. Accepting
it is.**

1. **The merge is the authority boundary.** An actor may draft, revise, and
   open a pull request of any scope its intent covers. It may not merge to a
   default branch, and merging is J0 in all cases — no rung below J0
   authorises it, and no standing authorization extends to it.
2. **Scope of drafting follows the intent, not the artifact's size.** A full
   rewrite of an always-on doctrine file is not a higher rung than a one-line
   fix merely because it is larger. What raises the rung is whether the
   *content* was ruled on, not how much of it there is.
3. **Undelegated choices inside a draft do not become delegated by being
   drafted.** Where a rewrite requires resolving something no ruling covers,
   the actor does not resolve it in prose. It records the choice as an open
   question in the PR body with a recommendation, or marks it
   **undelegated — parked**.
4. **Every PR whose content exceeds a single mechanical change carries a
   decision log** — `| Decision | Rung | Evidence | Resolution |` per
   `bounded-judgment.md`'s Decision evidence section. This is what makes
   approval a review of decisions rather than an audit of paragraphs, and it
   is the mechanism that keeps (3) honest.
5. **This narrows nothing about destructive or irreversible action.** Force
   pushes, history rewrites, deletion of preserved state, and secret
   exposure remain outside this ADR entirely — they are not "proposals" in
   any sense and are governed by `AGENTS.md`'s guardrails.

## Alternatives considered

- **Require pre-approval for large drafts.** Rejected: it makes dispatch
  incoherent, since every Work produces a branch, and it collapses the
  anti-capture separation from the worker's side — a worker declining to
  produce work is not independence.
- **Treat the PR itself as the decision and require the actor to stop
  short.** Rejected for the same reason, and because it produces worse
  artifacts: a half-drafted proposal is harder to rule on than a complete
  one with its open questions named.
- **Leave it to per-workflow authority envelopes.** Rejected: the question
  is general to all dispatch, and leaving it per-package is how the same
  ruling gets re-derived inconsistently in twenty places.

## Consequences

- Dispatched Works gain a clear, general answer to "how far may I go?" that
  does not require reading the delegating package for an implied limit.
- Review load shifts from prose auditing to decision-log adjudication. This
  is the point, but it is only cheaper if the decision log is honest —
  a log that omits the hard choices makes review *worse* than reading the
  diff, because it manufactures false confidence.
- The rule is silent on review quality. A finished artifact is a strong
  anchor, and "reject this PR" costs more than "choose between these." That
  is a real cost of allowing full drafts, and it is accepted rather than
  solved here.
- `bounded-judgment.md` needs a corresponding clarification so the ladder
  and this ADR do not diverge — the failure mode this repository calls
  instruction fiction.

## Open questions

- Does this extend to a PR against a non-default branch owned by another
  actor's in-flight Work? Currently unaddressed; the conservative reading is
  that writing into another Work's branch is not "proposing" and stays J0.
- What is the minimum decision-log threshold? "More than a single mechanical
  change" is a judgment call, and judgment calls about when to record
  judgment calls are self-referential in an unhelpful way.
