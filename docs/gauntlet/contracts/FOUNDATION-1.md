# FOUNDATION-1 — validate the foundation-rationalization proposal

Contract for a gauntlet unit whose artifact under review is a **proposal**,
not an implementation. Written by the orchestrating session (Captain) on
2026-08-14, per `reference/notes/gauntlet-pattern.md`.

## Artifact

`reference/proposal-foundation-rationalization.md`, §1–§8.

Companion records the panel may read as context, but is **not** grading:
`docs/adr/0005`–`0011` (the decisions), `docs/adr/0001`–`0004` (the
cross-platform decisions this builds on), and
`docs/gauntlet/runs/cross-platform-2026-08-14/` (the sprint that surfaced
most of the defects, its plan, and its lessons).

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
the foundation is sound with what is known now; it does not establish that
the plan is the right one to want. That distinction is the point of §8.1
below.

## Axes

Four blind critics, fresh context, one axis each, grading the proposal text
itself — never a summary of it.

1. **fidelity.** Does the proposal say what was decided, and *only* what was
   decided? Its seven changes are owner rulings recorded in ADRs 0005–0011.
   Flag any scope, rationale, consequence or commitment in the proposal that
   does not trace to one of those decisions or to cited evidence. Invented
   scope is the failure mode.

   Check specifically that three recorded **corrections of the orchestrating
   session** survive rather than being smoothed away: §5.3's brief-provoked
   loss of work, §5.5's rejected TUI carve-out, §5.7's argument-record-was-
   not-a-ruling. A proposal that reads as though Captain was right all along
   has been laundered.

2. **invariants.** Does any section violate `NORTH-STAR.md`'s ownership
   boundaries or its "Never" list, the R-NS-* rulings, the architecture
   invariants in `docs/DEVELOPMENT.md` (journal is the only truth, one
   owner, work state ≠ process state, ambiguity fails closed, disposable
   projections, adjacent-append hazard), or ADRs 0001–0011. **Includes the
   Ponytail Minimality Ladder** (`reference/notes/ideaos-agent-contract.md`):
   every addition should sit on its lowest viable rung; an unjustified R7 or
   a skipped rung is a finding. §4 claims these are preserved — grade the
   claim, do not accept it.

3. **assumptions.** Is every factual claim true? The proposal asserts file
   paths, line numbers, ruling ids, issue numbers, measured figures, and
   statements about what the code does today. Verify them against the
   repository as it stands. A false assumption is the most dangerous defect
   here, because the plan is built on it. Report each as claim → what you
   found → whether the section survives the correction.

4. **enactability.** Can each section actually be executed as dispatched
   work? A section is unenactable if it hides an undecided question behind
   confident prose, has no acceptance criterion a Work could satisfy,
   depends on something that does not exist, or contradicts another
   section's sequencing. §6's dependency order and §8's unknowns are
   in scope for this axis specifically.

## Method

Per `reference/notes/gauntlet-pattern.md`, with two adaptations recorded
here rather than made silently:

- **Critics run as dispatched sgt Works, not ultracode workflows.** The
  pattern predates the engine being able to run work; the model spread it
  fixes (executes vs. judges) is preserved — Sonnet critics, adjudication by
  the orchestrating session.
- **`test-honesty` is replaced by `enactability`.** The original axis grades
  whether tests verify their claims; there is no implementation here. The
  equivalent failure for a proposal is a section that cannot be carried out.

Refuters are batched **per axis** — one refuter per axis verifies all of
that axis's findings, never having written the proposal. Capped at **two
panel rounds**; anything still open after that is adjudicated, not
re-looped. Residual confirmed-but-deferred findings go to the ledger
backlog, never silently dropped.

## Non-goals

- **Not grading an implementation.** None exists. A finding of the form
  "this is not implemented" is out of scope by construction.
- **Not re-litigating the seven decisions.** They are owner rulings made at
  an explicit confirmation gate. The panel's question is whether the
  proposal expresses them faithfully, soundly and executably — not whether
  they should have been made. A critic that argues a decision was wrong
  should say so as an explicit out-of-scope note, not as a finding.
- **Not designing the implementations.** "This section should be built
  differently" is only a finding if the proposal's own stated approach is
  unenactable or violates an invariant.
- **Not extending scope.** Adjacent problems the proposal does not claim to
  solve are not findings. §7 lists the non-goals it declares.

## Unknowns

Named honestly, per the contract convention, rather than forced into
certainty.

1. **The anchor is weaker than a normal gauntlet's.** A milestone gauntlet
   grades an implementation against a contract that predates it. Here the
   panel grades a proposal against ADRs and an interview transcript it
   cannot see. The axes are chosen to compensate — `assumptions` and
   `enactability` are checkable against the repository — but `fidelity` is
   anchored on the ADRs alone, and if those are themselves wrong the panel
   will not catch it.

2. **§8.2 is the assumption most likely to be wrong**, by the proposal's own
   admission: whether a gate Work reviewing another Work's output stays as
   adversarial as an external pipeline. The panel cannot measure this. It
   can only judge whether the proposal is honest that it is unmeasured.

3. **No critic has run against a proposal in this repo before.** Every prior
   gauntlet graded code. Whether the four axes produce useful findings on
   prose, or degrade into style commentary, is itself unmeasured — and worth
   recording in the ledger entry either way, since it determines whether
   this method is reusable.
