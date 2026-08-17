# SPLIT-1 — validate the product/workspace split proposal

Contract for a gauntlet unit whose artifact under review is a **proposal**,
not an implementation. Written by the orchestrating session (Captain) on
2026-08-17, per `reference/notes/gauntlet-pattern.md` and the FOUNDATION-1
and T-SERIES-1 precedents (`docs/gauntlet/contracts/FOUNDATION-1.md`,
`docs/gauntlet/contracts/T-SERIES-1.md`) — the two prior units to grade a
proposal rather than code. This is the third.

## Artifact

`reference/proposal-product-workspace-split.md`, §1–§13.

Companion records the panel may read as context, but is **not** grading:

- `docs/adr/0014-product-workspace-split-owner-rulings.md` — **the fidelity
  authority.** Every decision in the proposal must trace here or to cited
  evidence. The owner rulings it records were made in conversation on
  2026-08-17; that conversation is not available to the panel, and this ADR
  is deliberately the only durable record of it.
- `NORTH-STAR.md`, including its 2026-08-17 dated amendments. Those rulings
  are themselves out of scope — grade the proposal, not whether the gate
  should have been lifted.
- `docs/DEVELOPMENT.md`, `docs/adr/0001`–`0013`, `LESSONS.md`,
  `.sergeant/common/contexts/bounded-judgment.md`, `.sergeant/index.md`.
- `~/inbox/proposal-ci-cd-release-engineering.md` (2026-08-16) — the
  re-scoped source. Still an inbox item; not accepted.

## Model and seats

Per ADR 0014 decision 13, narrowing R-S0-13 for this workstream: **critic
and refuter seats run on the `sonnet` profile. Captain (Opus) adjudicates.
No Fable seat.** Each critic gets fresh context and one axis. No critic sees
another's findings before its own are recorded.

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
the plan is sound and internally consistent with what is known now. It does
not establish that this is the reorganization to want, and it authorizes no
phase of the work. Phase 0 proceeds in parallel by separate authority (ADR
0014; the proposal's §6 scopes it as independent of the verdict).

**Captain wrote the artifact under review.** That is why this unit exists in
this form rather than as an in-session read-through: the proposal's own §4.6
holds that whoever defines "did it work" must not be whoever did it, and
that a proposer may never be sole approver. This contract is that rule
enacted. A critic that finds Captain's own reasoning congenial has probably
not done the job.

## Axes

Four blind critics, fresh context, one axis each, grading the proposal text
itself — never a summary of it.

### 1. fidelity

Does the proposal say what was decided, and *only* what was decided?

Every decision in §13's register, every phase in §6, and every doctrine
ruling in §4 must trace to ADR 0014, to an earlier ADR, or to cited
evidence. **Invented scope is the failure mode.** Flag any commitment,
rationale, or consequence with no traceable source.

Check specifically that four recorded corrections of the orchestrating
session survive rather than being smoothed away:

- Captain proposed **Notion as the project's knowledge system of record**
  and was corrected; the proposal must not read as though this was never
  entertained.
- Captain proposed **three repositories and extracting the distro out of
  the dev repo**; the owner inverted the dependency.
- Captain **killed `sgt workflow diff` on R1 grounds**, then had to restore
  the underlying property (an edition marker) when the research showed a
  fork has no invalidation mechanism.
- Captain **reached for Gmail** when told a proposal was "in the inbox,"
  having not read `docs/environments/cerberus.md`'s inbox convention — an
  instance of the very retrieval failure the proposal diagnoses.

A proposal that reads as though Captain was right all along has been
laundered. ADR 0014 records all four; the proposal should not contradict it.

### 2. invariants

Does any section violate `NORTH-STAR.md`'s ownership boundaries or its
"Never" list, the R-NS-* rulings, the architecture invariants in
`docs/DEVELOPMENT.md` (journal is the only truth, one owner, work state ≠
process state, ambiguity fails closed, disposable projections,
adjacent-append hazard), ADRs 0001–0014, or BU-0109 (one developer per
installation)?

Includes the **Ponytail Minimality Ladder** applied to the proposal's own
additions. Phase 3 is the only new engine work in the plan — grade whether
it sits on its lowest viable rung, and whether §4.7's edition marker,
§3.3's scoped-retrieval partitioning, and Phase 5's validator are each
minimum-sufficient or reach past R6. §12's non-goals claim restraint;
grade the claim, do not accept it.

Specific hazards to test: does embedding the distro in the binary
(decision 1) collide with the platform boundary (ADR 0002) or the
manifest's authority over storage paths (ADR 0008)? Does the workspace
estate mounting `sergeant-rs` violate "`repos/` is a mount, never a
dev_root"?

### 3. assumptions

The proposal rests on measurements taken in one session by the party that
wrote it. Grade them.

- **Reproduce the counts.** 216 template files; 162 contamination
  candidates; 91 hard references into `reference/sergeant-upstream/`; 611
  `BU-####` citations across 89 files; 80 stage `CONTEXT.md` files, 82
  bounded-judgment blocks, 77 citing `@@bounded-judgment`; three unresolved
  `@@refs`. Run the greps. Report any that do not hold.
- **Grade the 162 as an upper bound.** The proposal concedes it needs
  triage. Sample it: how many are genuine state contamination versus a
  legitimate Rust example? If the real number is small, §2.2's argument
  weakens and Phase 2's position as a prerequisite is in question.
- **Six Memory Failure Modes scoring RED six for six** (§2.1). Is each
  score defensible from the cited evidence, or is it motivated reasoning
  toward a conclusion the author had already reached? A score that cannot
  survive one counter-example is a finding.
- **The procedure/state test.** Does it actually partition 216 files, or
  are there templates that are neither cleanly procedure nor cleanly state?
- **The three arrival-gap incidents** (LESSONS L20, the `pkill`
  double-kill, the CI proposal's missed gate). Do the cited records say
  what the proposal claims they say?

### 4. sequencing

Do the gates hold, and is the authority split honest?

- Can Phase 2 (template decontamination) genuinely precede Phase 3
  (override mechanism and embedding), or does decontamination need the
  embedding format settled first?
- Is Phase 5's cross-repo skew check actually blocked on Phase 4, or could
  it run earlier against a working tree?
- §9's overnight scope assigns rungs (J4 / J2 / J0) to specific work.
  Grade each assignment against `.sergeant/common/contexts/bounded-
  judgment.md`. **Does §9 authorize more than it admits?** In particular:
  is "AGENTS.md rewrite → PR for morning review" really J2, given that
  §4.1 deletes an entire section of the always-on file?
- §4.6's tripwire (any J0 → J2 move is an explicit reviewed specification
  change) — does the proposal itself comply, or does §9 lower a rung by
  assertion?
- §7's gates: is any gate green-able by the same party it is meant to
  constrain?

## Refutation

Every finding from every axis goes to an independent refuter (fresh
context, `sonnet`) instructed to **refute it**, defaulting to refuted when
uncertain. A finding survives only if the refuter cannot break it. Refuters
do not see the critic's confidence or the other axes.

## Adjudication

Captain (Opus) adjudicates surviving findings into the three-state outcome
above, writes `docs/gauntlet/runs/split-1/adjudication.md`, and appends a
`GAUNTLET.md` ledger entry. Every material adjudication decision records
its J-rung per ADR 0013.

Findings that would change an owner ruling in ADR 0014 are **not** Captain's
to adjudicate — they escalate as J0 with a recommendation, per the
bounded-judgment ladder's own conflict rule.

## Run record

`docs/gauntlet/runs/split-1/` — one file per critic, one per refuter, plus
`adjudication.md`. Per `reference/notes/gauntlet-pattern.md`, the run
record is written as it happens, not reconstructed afterward.
