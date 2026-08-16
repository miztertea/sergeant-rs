# T-SERIES-1 — validate the revised T-series operator-cockpit proposal

Contract for a gauntlet unit whose artifact under review is a **proposal**,
not an implementation. Written by the orchestrating session (Captain) on
2026-08-15, per `reference/notes/gauntlet-pattern.md` and the FOUNDATION-1
precedent (`docs/gauntlet/contracts/FOUNDATION-1.md`,
`docs/gauntlet/runs/foundation-1/`), the first and so far only prior unit to
grade a proposal rather than code.

## Artifact

`reference/proposal-tui-t-series.md`, §1–§25 (2026-08-15 revision,
superseding the 2026-08-11 predecessor per its own §3 and front-matter
`supersedes` block).

Companion records the panel may read as context, but is **not** grading:
`NORTH-STAR.md` (including its 2026-08-15 dated amendment lifting the
T-series unblock condition — that ruling itself is out of scope for this
panel; grade the proposal, not whether it should have been unblocked),
`docs/DEVELOPMENT.md`, `docs/icm/convention.md` §2a (the workflow/CLI-surface/
operator-skill three-way test the proposal's §6 leans on), ADRs 0009–0011
(observation-never-materializes-the-daemon, bare `sgt` as homepage, dashboard
deletion), and `docs/gauntlet/runs/cross-platform-2026-08-14/` (the sprint
whose foundation work this proposal consumes).

## Bounded outcome

The proposal is graded on four axes by blind critics, every finding is
refuted adversarially, and what survives is adjudicated. The unit ends with
the proposal in one of three states, recorded in `GAUNTLET.md`:

- **validated** — no confirmed finding on `invariants` or `assumptions`
  survives refutation;
- **validated with findings** — surviving findings are recorded and either
  fixed in the proposal or carried as named open questions;
- **sent back** — a confirmed finding invalidates a section's premise, and
  the proposal is revised before enactment.

**Acceptance is the owner's, not the panel's.** This unit establishes that
the proposal is sound and current with what is known now; it does not
establish that this is the interface to want, nor does it authorize any
build. Build against the graded proposal is a separate, later contract
(§20's T0–T4 program shape, sequenced only after this unit closes).

## Axes

Four blind critics, fresh context, one axis each, grading the proposal text
itself — never a summary of it.

1. **fidelity.** Does the proposal say what the repository actually
   contains and decided, and *only* that? It claims to consume the shipped
   MVP, WATCH, Estate, `completed_dirty`, dashboard deletion, and the
   integration branch behind PR #111 — verify each consumed surface is
   represented as it actually ships, not as an idealized or aspirational
   version of it. §23 (Dispositions) claims specific decisions were
   "adopted," "revised," or "rejected" from the 2026-08-11 predecessor —
   check each disposition against that predecessor's actual text rather
   than trusting the label.

2. **invariants.** Does any section violate `NORTH-STAR.md`'s ownership
   boundaries or its "Never" list, the R-NS-* rulings (R-NS-6 especially —
   §5.6 "execution is not dialogue" is the proposal's own restatement of
   it, grade the restatement, don't accept it), the architecture invariants
   in `docs/DEVELOPMENT.md` (journal is the only truth, one owner, work
   state ≠ process state, clients are equal — reach state only through the
   API, ambiguity fails closed), or the equal-client boundary
   `tests/m6_surfaces.rs` t5 enforces today. **Includes the Ponytail
   Minimality Ladder** (`reference/notes/ideaos-agent-contract.md`): every
   addition should sit on its lowest viable rung; §22's decision register
   claims a rung for every normative decision — an unjustified R7, a
   skipped rung, or a register entry that doesn't match its own §-cited
   decision is a finding.

3. **assumptions.** Is every factual claim true *as of this session*, not
   as of the proposal's own audit timestamp? The proposal cites specific
   commits, PR numbers, issue numbers, file paths, line numbers, and
   measured figures. Verify them against the repository as it stands right
   now. **Named starting point, not the only check:** §12.4 treats PR #111's
   retained/reap surfaces as conditional on that PR merging — `gh pr view
   111` shows it is **already merged**. Find every place in the proposal
   this conditional appears and grade whether the section's behavior still
   holds unconditionally, or whether the proposal's own hedge ("if the
   integration branch's retained/reap surfaces merge") is now stale prose
   that should read as fact. A false or stale assumption is the most
   dangerous defect here, because sections are built on it. Report each as
   claim → what you found → whether the section survives the correction.

4. **enactability.** Can each section actually be executed as dispatched
   work? A section is unenactable if it hides an undecided question behind
   confident prose, has no acceptance criterion a Work could satisfy,
   depends on something that does not exist, or contradicts another
   section's sequencing. §20's T0–T4 phases and §21's acceptance contract
   are in scope for this axis specifically — can T0 actually be dispatched
   as the first `sgt run` against this proposal without further judgment
   calls the proposal itself should have made?

## Method

Per `reference/notes/gauntlet-pattern.md`, with the same two adaptations
FOUNDATION-1 recorded:

- **Critics run as dispatched `sgt` Works, not ultracode workflows.** The
  model spread it fixes (executes vs. judges) is preserved — Sonnet
  critics, adjudication by the orchestrating session (Captain).
- **`test-honesty` is replaced by `enactability`.** No implementation
  exists yet; the equivalent failure for a proposal is a section that
  cannot be carried out.

Refuters are batched **per axis** — one refuter per axis verifies all of
that axis's findings, never having written the proposal. Capped at **two
panel rounds**; anything still open after that is adjudicated, not
re-looped. Residual confirmed-but-deferred findings go to the ledger
backlog, never silently dropped.

## Non-goals

- **Not grading an implementation.** None exists. A finding of the form
  "this is not implemented" is out of scope by construction.
- **Not re-litigating the North Star amendment.** The 2026-08-15 dated
  ruling lifting the T-series gate was made in dialogue with the owner this
  session; the panel's question is whether the *proposal* is sound and
  current, not whether the gate should have been lifted. A critic that
  disagrees with the gate ruling should say so as an explicit out-of-scope
  note, not as a finding against this proposal.
- **Not designing the implementation.** "This section should be built
  differently" is only a finding if the proposal's own stated approach is
  unenactable or violates an invariant.
- **Not extending scope.** Adjacent problems the proposal does not claim to
  solve are not findings. §6.2 lists the non-goals it declares.

## Unknowns

Named honestly, per the contract convention, rather than forced into
certainty.

1. **This is the second proposal-grading unit ever run.** FOUNDATION-1
   established that critics find real defects in prose (13 findings, 12
   confirmed) rather than degrading into style commentary. Whether that
   held once, on a ~500-line proposal, generalizes to this one (~2,300
   lines, materially larger surface area — Home, Fleet, Workflows, Estate,
   the canonical Work surface, responsive composition, and a full visual
   system) is itself unmeasured by this unit's start.
2. **`fidelity`'s anchor is the shipped repository, not an ADR set.**
   FOUNDATION-1's fidelity axis checked the proposal against ADRs 0005–0011,
   a fixed, already-adjudicated target. This proposal's fidelity axis
   instead checks against "what MVP/WATCH/Estate/dashboard-deletion
   actually ship" — a moving, larger target the critic must independently
   verify in the codebase rather than read off a decision record. This is
   closer to `assumptions` than FOUNDATION-1's `fidelity` was; some overlap
   between the two axes is expected and not itself a defect.
3. **§8 (Visual System and Ratatui Feasibility) and §19 (Testing and
   Validation) are the sections most likely to hide unenactable claims**
   about what `ratatui`/`crossterm` can actually render or what a pure-state
   test can actually assert without a live daemon — the proposal's own
   Ponytail register (§22) is the compensating mechanism, but whether it
   is honest about the rungs it claims is exactly what the invariants axis
   must check, not assume.
