# ICM-R0 — validate the ICM-R procedure-authority proposal

Contract for a gauntlet unit whose artifact under review is a **proposal**,
not an implementation — same shape as FOUNDATION-1
(`docs/gauntlet/contracts/FOUNDATION-1.md`), reused here per that unit's own
precedent for proposal-only grading. Written by the orchestrating session
(Captain) on 2026-08-16, at the owner's direction: vendor the proposal from
the inbox, then validate it against a current commit before any owner
ruling on its §19 decisions.

## Artifact

`reference/proposal-icm-r-procedure-authority.md`, in full (§1–§20 plus the
Source-to-Decision Map and Owner Decisions).

**Audit-pin drift is in scope, not incidental.** The proposal states its own
audit basis as `3a46b87c17d249655708ed5ac32f6704738776cf` (the Path-to-Mac
merge, 2026-08-15). Current `main` is `ad20ec7` (2026-08-16, the full
T-series build + follow-up sprint), and this gauntlet itself runs from
branch `chore/backlog-grooming-2026-08-16` at `9e0b119`. Every factual claim
about the repository — workflow/skill counts, file paths, line-level
citations, "no code prerequisite," the current decomposition ladder's exact
text — must be checked against **current `main`**, not against the
proposal's stated pin. Where current `main` differs from what the proposal
describes, that is a fidelity or assumptions finding, not something to
silently wave through because the proposal was accurate as of its own
audit revision.

## Bounded outcome

Graded on four axes by blind critics, every finding refuted adversarially,
survivors adjudicated. Ends in one of three states, recorded in
`GAUNTLET.md`:

- **validated** — no confirmed finding on `invariants` or `assumptions`
  survives refutation;
- **validated with findings** — surviving findings recorded and either
  fixed in the proposal or carried as named open questions into the owner's
  §19 ruling;
- **sent back** — a confirmed finding invalidates a section's premise, and
  the proposal is revised before the owner rules on it.

**Acceptance is the owner's, not the panel's.** This unit establishes
whether the proposal's factual and structural claims hold against the
repository as it stands now. It does not adjudicate the twelve owner
decisions in the proposal's own §19 — those remain a live interview with
the owner, after this gauntlet, per `docs/gauntlet/promoted-provenance/`'s
grilling precedent (never dispatched, never decided by a panel).

## Axes

Four blind critics, fresh context, one axis each, grading the proposal text
itself — never a summary of it, never each other's findings.

1. **fidelity.** Does the proposal accurately represent the current ICM
   convention, decomposition ladder, workflow/skill catalog, and engine
   capabilities it claims to describe — checked against current `main`,
   not its own stated pin? §3 (Findings) makes seven specific claims about
   what the engine and ICM filesystem already do (ICMR-F1 through ICMR-F7);
   each is independently checkable against `src/runtime/engine.rs`,
   `src/backend/claude.rs`, `src/domain/workflow.rs`,
   `docs/icm/convention.md`, and `.sergeant/workflows/repo-to-icm/_config/
   icm-ladder.md`. §12's package hypotheses cite specific package contents
   (`task-intake-and-route`, `direct-implementation`, `sergeant-setup`,
   `dispatch`, `worker-mission`, `load-project`) — verify each package
   still contains what's described. Invented or stale scope is the failure
   mode.

2. **invariants.** Does any section violate `NORTH-STAR.md`'s ownership
   boundaries (Core/OS/Estate/Surfaces, R-NS-1..6), `AGENTS.md`'s routing
   doctrine, or any ADR now on `main` — **including ADR 0012** (2026-08-16,
   Estate/Doctor as daemon API surface), which postdates the proposal's own
   audit pin and is exactly the kind of "later engine capability" §2.2's
   foundation-rationalization citation says this catalog should be checked
   against. **Includes the Ponytail Minimality Ladder**
   (`reference/notes/ideaos-agent-contract.md`): every one of the proposal's
   own R1–R7 citations (§17's Ponytail Decision Register) sits on its
   claimed rung — an unjustified R7 or a skipped rung is a finding. Also
   check the proposal's own hard boundary (§ Executive Summary: "no changes
   to src/, API routes, journal schema, Work state, backend traits, TUI
   behavior, or workflow.toml grammar in the initial workstream") against
   everything §7 (Required Instruction Shapes) and §10.1–10.2 actually
   propose doing — a content change that quietly requires engine
   cooperation to be enforceable is an invariants violation of the
   proposal's own stated boundary, not just a documentation nit.

3. **assumptions.** Is every factual claim true, checked against current
   `main`? The proposal asserts a 23-workflow, 4-skill catalog (§ Executive
   Summary point 4, §10.4); specific file paths and line-level behavior
   (§2.3, §18's Source-to-Decision Map); that `validate-and-ship/
   40-drive-gates` already distinguishes auto-fix/no-op/ask-user (§3.4,
   §18); and that no PL-/J- rung vocabulary exists elsewhere in the repo
   (implied by ICMR-04's "extend, don't replace" framing). Verify the exact
   current workflow and skill counts (`.sergeant/index.md`,
   `.sergeant/workflows/`, `skills/`), and specifically check whether
   anything shipped in the T-series sprint (2026-08-16, PR #131 + #155–158)
   changes any cited fact. Report each as claim → what you found → whether
   the section survives the correction.

4. **enactability.** Can ICM-R0 through ICM-R4 actually be executed as
   dispatched work, or does a section hide an undecided question behind
   confident prose? §10 sequences five workstreams with stated outcomes —
   for each, check whether a dispatched Work could satisfy its stated
   outcome without inventing something the proposal should have supplied.
   §15's 38-item Acceptance Contract and §8.13's package-adjudication
   record shape are the checkable units — pick a sample and verify a
   dispatched actor could complete them from the proposal's own text alone.
   §19's twelve owner decisions are explicitly *not* enactable until ruled
   on — flag any place in §7–§10 that silently assumes a §19 decision's
   recommended default rather than treating it as pending.

## Method

Same as FOUNDATION-1: critics run as dispatched `sgt` Works (not the
`Workflow` tool) — Sonnet critics, adjudication by the orchestrating
session. This repo's engine can now run work reliably (T-series: 10/10
dispatched Works landed clean), so this is no longer a fallback forced by
an immature engine — it is dogfooding the product on its own procedure
review, which is the point.

Refuters are batched **per axis** — one refuter per axis verifies all of
that axis's findings, never having written the proposal. Capped at **two
panel rounds**; anything still open after that is adjudicated, not
re-looped. Residual confirmed-but-deferred findings go to the ledger
backlog, never silently dropped.

## Non-goals

- **Not ruling on §19.** Twelve owner decisions are named there explicitly
  as not silently made by this proposal. This gauntlet does not make them
  either — a critic that argues for one answer over another is out of
  scope; grade whether the *proposal* is honest that they're undecided.
- **Not grading an implementation.** None exists yet (ICM-R0 is explicitly
  no-code). A finding of the form "this isn't built" is out of scope.
- **Not designing an alternative ladder.** "This taxonomy should be
  different" is only a finding if the proposal's own stated ladder is
  internally inconsistent or violates a checked invariant.
- **Not extending scope.** Stranger-onboarding, T-series, P2-JOURNAL, and
  the adapter/release proposals are explicitly out of this proposal's
  relationship note (front matter) and out of this gauntlet too.

## Unknowns

1. **The audit-pin drift is itself the sharpest test of this proposal's own
   method.** ICM-R1's Ponytail decision ICMR-01 says "pin the audit
   revision; a proposal about source placement and package counts cannot
   design against an unqualified moving branch" — and one day later, main
   moved by an entire T-series sprint. Whether the proposal's claims
   survive that drift, or whether §4.11's "proposals are timestamped
   models" doctrine has to actually do work here, is exactly what this
   gauntlet measures.
2. **No critic has run against this proposal's own genre before.**
   FOUNDATION-1 was the first proposal-only gauntlet in this repo; this is
   the second, and the first one grading a proposal about *how proposals
   and packages should be classified* — a degree of self-reference worth
   naming rather than ignoring.
3. **Assumptions findings may cluster on T-series-era changes specifically**
   (ADR 0012, the T-series build's new API routes, the follow-up PRs'
   closure of #152/#153/#154) since those are the single largest concrete
   delta between the proposal's pin and current `main`.
