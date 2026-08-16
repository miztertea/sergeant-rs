# ICM-R0 — assumptions critic

Axis: **assumptions** (`docs/gauntlet/contracts/ICM-R0.md` axis 3).
Artifact: `reference/proposal-icm-r-procedure-authority.md`, read in full.

This is a blind review: no other critic's findings were read, and this
report was written before consulting anything else under
`docs/gauntlet/runs/icm-r0/`.

## Method

Checked against current `main` (`HEAD` at the time of this review:
`8a8959e`, 2026-08-16 — the ICM-R0 contract commit itself; the proposal's
own vendoring commit `9e0b119` is its immediate parent), not against the
proposal's stated audit pin `3a46b87c17d249655708ed5ac32f6704738776cf`
(2026-08-15, the Path-to-Mac merge), per the contract's explicit
instruction that audit-pin drift is in scope.

For each claim: read the proposal's exact wording, then verified directly
against the repository rather than trusting the proposal's own citations.

1. **Workflow/skill catalog counts** — counted `.sergeant/workflows/*`
   directories, `skills/*` directories, and `.sergeant/index.md`'s table
   rows independently, then cross-checked all three against each other and
   against the proposal's stated "23 workflows / 4 skills."
2. **File paths and citations (§2.3, §18)** — checked existence of every
   cited path in the Source-to-Decision Map (§18) and the "repository
   materials reviewed" list (§2.3), both at the proposal's stated pin
   (`3a46b87c...`) and at current `HEAD`.
3. **`validate-and-ship/40-drive-gates` auto-fix/no-op/ask-user claim
   (§3.4, §18)** — opened the stage's `CONTEXT.md` in full and checked
   whether the three-way finding classification the proposal describes is
   actually present, worded as described.
4. **No prior PL-/J- rung vocabulary (implied by ICMR-04)** — grepped the
   full repository (all `.md` files) for `PL-[0-9]` and `J[0-5]` used as a
   rung/delegation/judgment vocabulary, excluding the proposal itself.
5. **T-series drift** — diffed every file cited in §18 and §2.3 between the
   proposal's pin and current `HEAD` (`git diff --stat <pin>..HEAD -- <paths>`),
   then read the diffs for any file that changed, to determine whether the
   T-series sprint (PR #131, follow-ups #155–158) or ADR 0012 altered any
   fact the proposal asserts as current.

## Findings

### F1 — severity: info — Executive Summary point 4 / §10.4 (23-workflow, 4-skill catalog)

**Claim.** The proposal states the catalog to reconcile is "all 23
published workflows and four current operator skills" (Executive Summary
point 4; repeated verbatim in §10.4).

**What I checked.** Counted `.sergeant/workflows/*` directories (23),
`skills/*` directories (4), and `.sergeant/index.md`'s workflow table rows
(23 `published` rows, plus a prose line naming exactly four operator
skills: `sergeant-help`, `grilling`, `grill-with-docs`,
`estate-navigation`). All three sources agree with each other and with the
proposal, on current `HEAD`, not just at the proposal's stated pin.
`.sergeant/index.md` itself has zero diff between the pin and `HEAD`
(`git diff --stat <pin>..HEAD -- .sergeant/index.md` is empty), so this
count did not drift during the T-series sprint.

**What I found.** Claim is accurate. No correction needed.

**Verdict.** Survives.

### F2 — severity: info — §2.3 and §18 (file paths and citations)

**Claim.** §2.3 lists a specific set of repository materials reviewed
(NORTH-STAR.md, AGENTS.md, docs/icm/convention.md,
docs/icm/record-shapes.md, .sergeant/workflows/repo-to-icm/_config/
icm-ladder.md, three docs/icm dated records, the 23-workflow/4-skill
catalog, six representative packages, workflow/stage-execution
implementation, and the N4 draft). §18's Source-to-Decision Map cites 17
in-repo files by exact path (pinned to `3a46b87c...` in each GitHub URL)
plus several external sources.

**What I checked.** Verified every in-repo path named in §2.3 and §18
exists (a) at the stated audit pin `3a46b87c17d249655708ed5ac32f6704738776cf`
and (b) at current `HEAD`. Also diffed each cited file between the pin and
`HEAD` to see whether its content had moved since the URLs were minted.

**What I found.** All 17 paths exist at both revisions — none is stale or
invented. Of the cited files, only two changed at all between the pin and
`HEAD`: `NORTH-STAR.md` (a dated 2026-08-15 amendment lifting the T-series
build gate — unrelated to the Captain/dialogue-ownership claim the
proposal cites it for) and `src/domain/workflow.rs` (T-series T2 added a
purely additive `catalog()`/`CatalogEntry` read-only workflow-discovery API
for `GET /v1/workflows`; the pinned-stage-content/execution-reservation
code the proposal actually cites this file for, e.g. the `resolve` method
and the pinned-`ExecuteSpec`/stage-order structures, is untouched).
`src/runtime/engine.rs`, `src/backend/claude.rs`, `docs/icm/convention.md`,
`AGENTS.md`, and `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`
— the files carrying the specific behavior claims §18 attributes to
them — all show zero diff since the pin.

**Verdict.** Survives. The two changed files do not falsify anything the
proposal actually attributes to them; the new workflow-discovery API in
`workflow.rs` is a genuinely new capability that postdates the pin, but it
is additive and orthogonal to the cited behavior (stage pinning/reservation),
so it is not an assumptions-axis defect. (Whether the proposal *should* have
noticed and accounted for this new capability as "later engine capability"
in the way ADR 0012 is flagged for the invariants axis is a fidelity/
invariants question, not an assumptions one — the citation itself is not
false.)

### F3 — severity: info — §3.4 and §18 (`validate-and-ship/40-drive-gates` auto-fix/no-op/ask-user)

**Claim.** "`validate-and-ship/40-drive-gates`, for example, permits the
actor to handle auto-fix and no-op findings but explicitly reserves
ask-user findings for the user" (§3.4), restated in §18 as "existing
concrete bounded-judgment precedent."

**What I checked.** Read `.sergeant/workflows/validate-and-ship/
40-drive-gates/CONTEXT.md` in full at current `HEAD` (this file has zero
diff since the pin).

**What I found.** The claim is precisely accurate, not a rough
paraphrase. The stage's behavior contract states verbatim: a `gate:`
object's findings table classifies each finding's `action` as `auto-fix`
("mechanical/low-risk, actor may authorize on their own judgment"),
`no-op` ("informational, nothing to do"), or `ask-user` ("challenges the
user's deliberate intent or touches product behavior — a decision only the
user can make") (`BU-P2-079`). A later behavior unit restates the same
three-way split and explicitly forbids the actor from resolving an
ask-user finding autonomously (`BU-P2-098`, `BU-P1-075`). This is exactly
the three-way distinction and asymmetric authority the proposal
describes.

**Verdict.** Survives.

### F4 — severity: info — ICMR-04 framing ("extend, don't replace"; no PL-/J- vocabulary exists elsewhere)

**Claim.** ICMR-04 (§17 Ponytail Decision Register) resolves to "Extend the
existing decomposition method; do not create an unrelated classifier,"
which only makes sense as a non-duplication claim if no PL-N / J-N rung
vocabulary already exists elsewhere in the repository that this proposal
would be silently colliding with or duplicating.

**What I checked.** `grep -rn -E '\bPL-[0-9]\b'` and a pattern for `J[0-5]`
used near "rung," "delegat-," "judgment," or "decision" across every
Markdown file in the repository, excluding the proposal itself.

**What I found.** Zero matches outside the proposal. No prior PL-N or J-N
rung vocabulary exists anywhere else in the repository — not in
`docs/icm/convention.md`, not in `.sergeant/workflows/repo-to-icm/_config/
icm-ladder.md` (which uses a different, unnumbered first-match rung scheme
with named categories, not PL-/J- labels), not in any other proposal.

**Verdict.** Survives.

### F5 — severity: info — audit-pin identity and reachability

**Claim.** The proposal is "pinned to main at
`3a46b87c17d249655708ed5ac32f6704738776cf`, the merge of the Path-to-Mac
sprint on 2026-08-15" (§2.1).

**What I checked.** `git cat-file -t` and `git log -1 --format='%H %ci %s'`
on that hash; `git merge-base --is-ancestor` against current `HEAD`.

**What I found.** The hash resolves to a real commit dated 2026-08-15,
message "Merge pull request #111 from miztertea/integration/
path-to-mac-2026-08-15" — matching the proposal's description exactly. It
is an ancestor of current `HEAD`, i.e. a real point on this repository's
actual history, not a dangling or foreign hash.

**Verdict.** Survives.

## Summary

Every claim assigned to this axis — the 23-workflow/4-skill catalog count,
every file path and citation in §2.3 and §18, the `40-drive-gates`
auto-fix/no-op/ask-user precedent, and the absence of prior PL-/J- rung
vocabulary elsewhere in the repository — checks out against current `main`,
not merely against the proposal's own stated pin. The T-series sprint
(PR #131 and follow-ups #155–158) touched only two of the ~17 files this
proposal cites, and neither change falsifies what the proposal attributes
to that file. No finding on this axis invalidates any section's premise.
