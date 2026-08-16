# ICM-R0 — assumptions refuter

Axis: **assumptions** (`docs/gauntlet/contracts/ICM-R0.md` axis 3).
Critic report under review: `docs/gauntlet/runs/icm-r0/critics/assumptions.md`.
Artifact both were grading: `reference/proposal-icm-r-procedure-authority.md`.

I did not write the proposal or the critic report. For each finding I
re-derived the claim independently against the repository (`git`, `grep`,
file reads) rather than trusting the critic's own quotations, and argued
against the finding before accepting it.

## F1 — 23-workflow/4-skill catalog count

**Critic's claim.** `.sergeant/workflows/*` (23), `skills/*` (4), and
`.sergeant/index.md`'s 23 `published` rows plus four named operator skills
all agree with the proposal's "23 workflows / 4 skills," on current `HEAD`,
with zero diff on `index.md` since the pin.

**Attempted refutation.** A naive `grep -c "published"
.sergeant/index.md` returns **24**, not 23 — line 4 of that file is prose
("Lists every `status: published` workflow under...") that also contains
the word "published" and inflates a crude count by one. If the critic's
"23" came from an equally crude count, it would be right by coincidence
and wrong in method.

**Independent re-derivation.**
- `ls -d .sergeant/workflows/*/ | wc -l` → 23.
- `ls -d skills/*/ | wc -l` → 4.
- `grep -n "| published |" .sergeant/index.md | wc -l` → 23 (table rows
  only, excluding the prose line that trips a naive count).
- The four skill names are listed verbatim at `.sergeant/index.md:48-49`:
  `sergeant-help`, `grilling`, `grill-with-docs`, `estate-navigation`.
- `git diff --stat 3a46b87c..HEAD -- .sergeant/index.md` is empty — no
  drift since the pin.

All four independent counts agree with each other and with the proposal.
The critic's number is correct; only a sloppier counting method could have
produced a different answer, and the critic's own described method
(counting three independent sources and cross-checking) avoids that trap.

**Verdict.** CONFIRMED.

## F2 — §2.3 and §18 file paths and citations

**Critic's claim.** All 17 in-repo paths cited in §2.3/§18 exist at both
the stated pin (`3a46b87c...`) and current `HEAD`. Only `NORTH-STAR.md` and
`src/domain/workflow.rs` changed between the pin and `HEAD`; the change to
`workflow.rs` is purely additive (a new `catalog()`/`CatalogEntry`
read-only API) and does not touch the `resolve` method or
`ExecuteSpec`/stage-order structures the proposal actually cites it for.

**Attempted refutation.** "Purely additive" is exactly the kind of claim a
critic could get wrong by skimming a diffstat without reading the diff
body — an additive-looking diff can still shadow or shift behavior via
trait impls, re-exports, or default-parameter changes elsewhere in the same
file.

**Independent re-derivation.**
- Checked existence of all 8 sampled core paths (`NORTH-STAR.md`,
  `AGENTS.md`, `docs/icm/convention.md`, `docs/icm/record-shapes.md`,
  `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`,
  `src/runtime/engine.rs`, `src/backend/claude.rs`,
  `src/domain/workflow.rs`) at both the pin and `HEAD` via `git cat-file
  -e` — all present at both revisions.
- `git diff --stat 3a46b87c..HEAD` over that same set: only
  `NORTH-STAR.md` (+22 lines) and `src/domain/workflow.rs` (+415 lines,
  0 deletions) changed.
- Read the full `workflow.rs` diff: it adds `INDEX_FILE`,
  `ROOT_CATALOG_FILE` constants, new structs `WorkflowIndexFrontMatter` and
  `CatalogEntry`, a new `pub fn catalog(root: &Path)`, and private helpers
  plus tests — all new code, zero deletions, zero modifications to existing
  lines. The one call into existing code is
  `WorkflowDefinition::resolve(root, &name).ok()?` — a call site, not a
  change to `resolve` itself.
- Read the `NORTH-STAR.md` diff: a dated 2026-08-15 amendment lifting the
  T-series build gate, unrelated to Captain/dialogue-ownership content.

The diff is genuinely additive by content, not merely by diffstat shape.

**Verdict.** CONFIRMED.

## F3 — `validate-and-ship/40-drive-gates` auto-fix/no-op/ask-user

**Critic's claim.** The stage's `CONTEXT.md` states verbatim the three-way
auto-fix/no-op/ask-user finding classification and the asymmetric
ask-user-only-for-the-user rule, matching §3.4/§18's description exactly,
with zero diff since the pin.

**Attempted refutation.** "Precisely accurate, not a rough paraphrase" is a
strong claim — worth checking whether the proposal's summary elides a
qualifier the stage contract actually carries (e.g. the `--yes`
standing-consent override, which changes who can resolve an ask-user
finding).

**Independent re-derivation.**
- `git diff --stat 3a46b87c..HEAD -- .../40-drive-gates/CONTEXT.md` →
  empty, confirming zero drift since the pin.
- Read the file directly: `BU-P2-079` states the three-way
  auto-fix/no-op/ask-user classification verbatim, matching the proposal's
  wording almost word-for-word. `BU-P1-075` and `BU-P2-098` restate the
  same split and forbid autonomous resolution of ask-user findings.
- The file does carry a documented exception (`BU-P2-100`, `--yes`
  standing consent) that the proposal's §3.4 sentence does not mention.
  This is a real omission, but it is not a factual misstatement — the
  proposal's claim is about the *default* three-way split and the
  *general* asymmetric-authority rule, both of which hold; it does not
  assert "there is no override," so it is not falsified by the existence
  of one. This is at most a completeness nit, not an assumptions-axis
  defect, and the critic's own summary phrase "exactly the three-way
  distinction and asymmetric authority the proposal describes" is accurate
  to what §3.4 actually claims.

**Verdict.** CONFIRMED.

## F4 — no prior PL-/J- rung vocabulary elsewhere in the repo

**Critic's claim.** Zero matches for `PL-[0-9]` or rung/judgment-adjacent
`J[0-5]` usage outside the proposal, anywhere in the repository.

**Attempted refutation.** This is the finding most likely to be wrong by
omission, since a grep-based negative claim is only as good as its pattern
and exclusion list. I ran my own greps without reusing the critic's
described pattern, and found matches the critic's summary doesn't
mention:

- `docs/gauntlet/runs/icm-r0/critics/invariants.md` uses `PL-2`, `PL-7`,
  etc.
- `docs/gauntlet/runs/icm-r0/critics/enactability.md` uses `J0`, `J2`,
  `J5–J3` repeatedly.
- `reference/proposal-journal-query-p2.md` uses `J0`, `J1`, `J2` — and one
  instance (line 726) sits in the same sentence as the word "rung": "a
  derived `search_text` field may be considered at a later rung."

The first two are other ICM-R0 critic reports discussing this same
proposal — contemporaneous gauntlet artifacts, not pre-existing vocabulary
this proposal would be "silently colliding with or duplicating." They are
correctly out of scope for a "prior art" check and the critic's exclusion
of "the proposal itself" implicitly should extend to the gauntlet's own
review of it; I don't fault the critic for not calling this out, but it's
worth recording that "the proposal" search space is slightly narrower than
literally just the one file.

The third is substantive enough to check properly. Reading
`reference/proposal-journal-query-p2.md` in context: `P2-J0`, `P2-J1`,
`P2-J2` are section headers under "§19 Program Shape" —
`## P2-J0 — Adjudication and measured query spike`,
`## P2-J1 — Typed query engine and API`, `## P2-J2 — CLI, performance, and
documentation`. This is a **workstream/phase numbering convention**
(directly analogous to this proposal's own `ICM-R0`/`ICM-R1`/`ICM-R2`
phase identifiers), not a decision-authority rung vocabulary. The line-726
"rung" mention is Ponytail's generic ladder language ("a later rung"),
unrelated to a `J0`-`J5` bounded-judgment scale — it is describing a
future budget-negotiation step, not citing an authority level. This
committed 2026-08-11, before the proposal's own audit pin, so it long
predates ICM-R and is a genuine pre-existing use of "J-N" notation in the
repository — just not the *kind* of vocabulary (delegation/judgment rung)
the critic's check and ICMR-04's framing are actually about.

This is a real near-miss worth naming: the repository now has two
unrelated meanings for "J-N" tokens (P2's phase numbering vs. the
proposal's authority ladder). That is a legitimate terminology-collision
observation, but it does not falsify the critic's specific claim, which
was scoped to "rung/delegat-/judgment/decision" vocabulary — P2-J0/J1/J2
is phase numbering, not a judgment-authority scale, so it does not
duplicate or collide with what ICMR-04 is actually resolving against
("no unrelated classifier already exists").

**Verdict.** CONFIRMED, with a caveat recorded above (the P2-JOURNAL
`J0`/`J1`/`J2` phase-numbering usage is a near-miss on the search pattern,
worth a future terminology note, but does not constitute the kind of
rung vocabulary the finding is actually about).

## F5 — audit-pin identity and reachability

**Critic's claim.** `3a46b87c17d249655708ed5ac32f6704738776cf` is a real
commit dated 2026-08-15, "Merge pull request #111 from
miztertea/integration/path-to-mac-2026-08-15," and is an ancestor of
current `HEAD`.

**Attempted refutation.** None available — this is the easiest claim to
mechanically verify and hardest to get wrong; checked anyway for
completeness rather than skipped.

**Independent re-derivation.**
- `git cat-file -t 3a46b87c...` → `commit`.
- `git log -1 --format='%H %ci %s' 3a46b87c...` →
  `3a46b87c17d249655708ed5ac32f6704738776cf 2026-08-15 11:28:02 -0400
  Merge pull request #111 from miztertea/integration/path-to-mac-2026-08-15`
  — matches the critic's description exactly.
- `git merge-base --is-ancestor 3a46b87c... HEAD` → succeeds (is an
  ancestor).

**Verdict.** CONFIRMED.

## Summary

All five findings in `docs/gauntlet/runs/icm-r0/critics/assumptions.md`
survive independent adversarial re-derivation. None is struck or
downgraded. F1's undocumented crude-count trap and F4's P2-JOURNAL
near-miss are recorded above as things a less careful critic could have
gotten wrong, but in both cases the critic's actual claim holds under a
from-scratch check against current `main`. No finding on this axis
invalidates any section's premise; the assumptions axis has no surviving
material defect to carry into the owner's §19 ruling.
