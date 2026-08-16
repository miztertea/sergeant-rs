# ADR 0013: ICM-R0 owner rulings — Placement/Bounded-Judgment ladders accepted, pilot scoped

**Status:** Accepted, 2026-08-16.

## Context

`reference/proposal-icm-r-procedure-authority.md` proposes a Placement
Ladder (PL-0..PL-7) and Bounded-Judgment Ladder (J5..J0) to reconcile
Sergeant's 23-workflow/4-skill procedural library behavior-unit by
behavior-unit, content-only through its ICM-R4 workstream. It was
submitted by the owner via the inbox convention, vendored into
`reference/`, and graded by the ICM-R0 gauntlet
(`docs/gauntlet/contracts/ICM-R0.md`, `docs/gauntlet/runs/icm-r0/`):
validated with findings — two confirmed error-severity findings, both
narrow, neither invalidating the proposal's architecture (full record in
`docs/gauntlet/runs/icm-r0/adjudication.md`).

The proposal's own §19 names twelve owner decisions it explicitly does not
make on its own authority. This ADR records the live interview that ruled
on all twelve, conducted directly with the owner per `skills/grill-with-
docs/SKILL.md` (one question at a time, orchestrating session's own
recommendation offered alongside each, no action before explicit
confirmation of every decision).

## Decisions

1. **Names.** Accepted as written: "Placement Ladder (PL)" and
   "Bounded-Judgment Ladder (J)." No existing collision in the repo
   (checked directly, and independently reconfirmed by the ICM-R0
   assumptions critic/refuter pair).
2. **Skill taxonomy.** Captain skills and actor skills share the existing
   `skills/` root; a `driver:` front-matter field distinguishes them, not
   separate subdirectories. Matches how `.sergeant/workflows/` already
   mixes stage kinds under one flat root.
3. **Universal scope.** Shared contexts and helpers are first-class review
   subjects now. The built-in `software-change` workflow (`src/workflows/
   software-change/`) is explicitly **excluded** from this workstream's
   universal-scope claim — reconciling it as written would edit `src/`
   during ICM-R0–R4's own declared freeze (invariants F1). It gets its own
   later pass. Owner direction, same session: `software-change` should
   also be renamed (candidate: `default`/`fallback`) to make its
   generic-engine-fallback role unambiguous — a separate, deferred task,
   not a reason to skip evaluating its content eventually.
4. **Stage requirement.** Every actor stage carries an explicit local
   `## Bounded judgment` section always, even when it is only "inherits
   workflow envelope unchanged." Omission is never ambiguous.
5. **Decision recording.** Only *material* J2 decisions are recorded,
   matching the ladder's own definition (§6.1) rather than hedging beyond
   it.
6. **Generated-output invariant.** Independent review is required only for
   artifacts that will be promoted, merged, published, installed,
   admitted, signed, released, or treated as settled — not for every
   generated file. Ephemeral output (e.g. this gauntlet's own critic
   reports, which fed an adjudication but were never independently
   reviewed themselves) does not require review-of-the-review.
7. **Review independence.** A later stage in the *same* workflow may
   qualify as independent when it has a fresh execution, explicit inputs,
   a review-only contract, and no edit authority. Independence lives in
   the execution boundary, not the workflow-packaging boundary. Ratifying
   this resolves enactability F3 (§9.5 had stated it as settled while it
   was still pending).
8. **Pilot corpus.** The proposal's seven packages (`grilling`,
   `sergeant-help`, `task-intake-and-route`, `sergeant-setup`,
   `validate-and-ship`, `code-review`, `repo-to-icm`) are accepted, plus
   **`research`** is added — evidenced directly by this gauntlet's own B9
   finding (a dispatched `research` Work navigated out of its assigned
   worktree instead of stopping to ask; see Backlog table, `GAUNTLET.md`).
   The pilot should give `research` a real stage-level Bounded-judgment
   section, including an explicit surface-boundary clause, as a direct
   test of whether the ladder prevents a failure already observed rather
   than a hypothetical one.
9. **Likely rehomes.** All three of the proposal's most disruptive
   hypotheses (`task-intake-and-route`, `sergeant-setup`,
   `direct-implementation`) are tested in the pilot now, not deferred to
   "safer" packages first. **`direct-implementation` is added to the pilot**
   (it was named in this decision's own question but absent from §10.3's
   seven) — the pilot is now nine packages:
   `grilling`, `sergeant-help`, `task-intake-and-route`, `sergeant-setup`,
   `validate-and-ship`, `code-review`, `repo-to-icm`, `research`,
   `direct-implementation`.
10. **Runtime freeze.** "No Rust/runtime changes through ICM-R4" is a hard
    contract for this workstream, no urgency exception. Urgent runtime
    defects remain separate work through the normal bug-fix path.
    Ratifying this resolves enactability F4 (the same silently-assumed-
    then-ratified pattern as decision 7/F3).
11. **Proposal placement.** Confirmed as already executed:
    `reference/proposal-icm-r-procedure-authority.md` plus
    `docs/gauntlet/contracts/ICM-R0.md`, following house convention. The
    source file has been removed from `~/inbox/` per the inbox convention
    (only not-yet-accepted material stays there).
12. **Review workflow names.** Not pre-created. Names such as
    `review-document`/`review-pr`/`review-skill`/`review-workflow` are
    derived during the pilot's synthesis step if the corpus actually earns
    them, not invented in advance — the same file-shape-mirroring failure
    §8.8 already warns against.

## Framing carried forward, not a numbered decision

The owner's closing framing for this ruling: the current 23-workflow/4-
skill corpus is understood as **reference material to reconcile against**,
not settled procedure — consistent with the proposal's own Findings
ICMR-F5/F6 (structural promotion proved execution order, not procedural
truth). The sequence is: understand what capabilities and work-types
actually exist and need procedure, then build the workflows/skills proper
— ICM-R1 onward, not this ADR.

## Consequences

ICM-R1 (doctrine, templates, pilot instruments) is unblocked by this
ruling. The nine-package pilot (decisions 8–9) is ICM-R2's subject. The
two corrections named in the ICM-R0 adjudication (§10.4's `software-change`
exclusion, now formalized by decision 3; §10.1's "adjudicate" →
"surface... for adjudication" wording fix) should land in the proposal
text as part of ICM-R1, alongside recording these twelve rulings in the
proposal itself so it stops citing them as open.
