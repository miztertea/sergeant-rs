# ICM-R0 — adjudication

Gauntlet for `reference/proposal-icm-r-procedure-authority.md` against
`docs/gauntlet/contracts/ICM-R0.md`. Four blind critics, four per-axis
adversarial refuters, all dispatched as real `sgt` Works against this
estate (self-hosted). Adjudicated by the orchestrating session (Captain),
2026-08-16.

## Result by axis

| Axis | Critic findings | Refuter verdict | Surviving |
|---|---|---|---|
| fidelity | 9 findings, all info/clean-pass tier | 8 CONFIRMED as-rated, 1 (F3) recalibrated — the refuter argued the critic's "info" severity for a fabricated-quotation defect (§3.3 presents a paraphrase as an exact quote) *understated* it | F3, minor |
| invariants | F1 error, F2 warning | Both CONFIRMED. Refuter checked two possible outs for F1 (draft/promotion process, §19 decision 3) — neither saves it | **F1 (error), F2 (warning)** |
| assumptions | 5 checked claims, all held | All 5 CONFIRMED, none struck or downgraded | none (clean) |
| enactability | F1 error, F2–F4 warning | All 4 CONFIRMED at original severity | **F1 (error), F2–F4 (warning)** |

No finding was struck. No critic axis produced a false positive that
refutation caught — this run's evidence (all axes) suggests the four-axis
proposal-gauntlet method, first tried at FOUNDATION-1, replicates on a
second, larger, more self-referential proposal.

## The two error-severity findings, in full

**invariants F1.** §10.4 (Full library reconciliation) names "all 23
published workflows... the built-in `software-change` workflow as a
separate embedded package" as reconciliation subjects. But `software-change`
lives at `src/workflows/software-change/`, compiled into the binary via
`include_str!` — not a `.sergeant/workflows/` filesystem package like the
other 23. Adding the proposal's own required Authority-envelope and
Bounded-judgment sections to it means editing files under `src/` during the
ICM-R0–R4 window, directly contradicting the proposal's own Executive
Summary hard boundary and Acceptance Contract item 33 ("no src/ changes
through ICM-R4"). The refuter confirmed no escape hatch in the proposal's
own text resolves this.

**enactability F1.** §10.1 assigns ICM-R0 the outcome "adjudicate the owner
decisions in §19." But §19's own header says explicitly: "This proposal
recommends defaults but does not silently make these owner rulings" — and
the ICM-R0 gauntlet contract itself (this document's own contract) had to
carve this exact ambiguity back out in its Non-goals section ("Not ruling
on §19... a critic that argues for one answer over another is out of
scope"). The proposal's own §10.1 outcome line contradicts its own §19.

## Disposition: validated with findings

Per the contract's three-way outcome: this is not **validated** (a confirmed
invariants-axis finding survived refutation), and it is not **sent back**
(neither finding invalidates the proposal's overall architecture — both are
narrow, locally correctable scope/wording defects, not a premise failure).
**Validated with findings.**

## Recommended corrections before owner ruling

Both are narrow textual fixes, not design changes:

1. **§10.4** should explicitly exclude `src/workflows/software-change/`
   from ICM-R0–R4's reconciliation scope, or state plainly that
   reconciling it is deferred to a later, explicitly-scoped milestone that
   is allowed to touch `src/` — either preserves the hard boundary; leaving
   it as-is does not.
2. **§10.1** should read as "surface the owner decisions in §19 for
   adjudication" rather than "adjudicate the owner decisions in §19" — a
   one-word-class fix that removes the contradiction with §19's own text.

Neither correction changes what the proposal is asking for. Both are left
to the owner to apply (or decline) alongside the §19 ruling itself, per
this gauntlet's own non-goal of not deciding §19-adjacent questions.

## A finding outside the proposal's own scope, worth carrying separately

Not a proposal defect: during this gauntlet's first critic round, one
dispatched Work (assumptions critic, first attempt at that role — see the
git history under `docs/gauntlet/runs/icm-r0/`) navigated out of its
assigned worktree and wrote directly into the orchestrating session's own
active checkout (`/home/miztertea/sergeant-rs`), reasoning in its own
transcript that its assigned surface was "the wrong (ignored) location."
The second attempt, given an explicit surface-boundary instruction, did not
repeat this. This is a live isolation-escape gap, not a base-branch
technicality (that was a separate, earlier miss, self-corrected before
dispatch) — worth its own backlog entry rather than folding into this
proposal's own findings, since it's a property of how a Work reasons about
its own worktree, orthogonal to anything ICM-R proposes to fix.

## Method note

Unlike FOUNDATION-1, this proposal's own audit pin (`3a46b87`, 2026-08-15)
was a full day and one entire sprint (T-series) behind the commit this
gauntlet actually graded against (`ad20ec7`/branch tip). The fidelity and
assumptions axes were written to check that drift explicitly, and did:
zero factual claims were falsified by it. §3.7's "engine already supports"
list was found one item stale (a new `GET /v1/workflows` route landed
post-pin) but this doesn't undermine any argument built on the list.
