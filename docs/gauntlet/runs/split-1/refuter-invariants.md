# SPLIT-1 — refuter report, axis: invariants

Fresh context, `sonnet` profile. Attacking each numbered finding in
`docs/gauntlet/runs/split-1/critic-invariants.md` against
`reference/proposal-product-workspace-split.md`, `docs/adr/0014-product-
workspace-split-owner-rulings.md`, `docs/DEVELOPMENT.md`, `reference/notes/
ideaos-agent-contract.md`, ADR 0002, ADR 0008, and two prior gauntlet-graded
proposals (`reference/proposal-tui-t-series.md`,
`reference/proposal-icm-r-procedure-authority.md`). Did not read the other
three critic files or any other refuter's output. Default posture: refute
unless the finding survives independent verification.

## Finding 1 — Phase 3 carries no Ponytail rung

**Verdict: SURVIVES.**

Tried to break this on the theory that the rung-logging convention binds only
*already-implemented* work recorded in `GAUNTLET.md` — i.e., that a proposal
document, being pre-decision, is exempt and the rung gets logged later when
Phase 3 actually ships. `docs/DEVELOPMENT.md:100` and
`reference/notes/ideaos-agent-contract.md:50-54`'s own wording ("every design
decision in a ledger entry, every deviation-register row, and every new
dependency, file, trait, or store") textually supports that reading — a
proposal is none of the three.

That refutation does not survive contact with this repository's own
precedent. I read the two prior gauntlet-graded *proposal* artifacts named in
the SPLIT-1 contract's lineage:

- `reference/proposal-tui-t-series.md:61`: "Every normative decision names
  its lowest viable Ponytail rung. The complete register is §22" — written
  while `status: proposed`, before T0–T4 were built.
- `reference/proposal-icm-r-procedure-authority.md:1584-1596`, §17 "Ponytail
  Decision Register" — an eleven-row table (`ICMR-01`...`ICMR-11`) each with
  an explicit rung, also written pre-implementation.

Both precedents put the rung register **in the proposal itself**, not
deferred to a later ledger entry. SPLIT-1's proposal contains zero rung
citations anywhere — verified by grep: no `R1` through `R7` token appears in
`reference/proposal-product-workspace-split.md` at all. Against the
convention's letter this is arguable; against this project's own practice
for artifacts of exactly this kind, it is a real, checkable gap, and the
critic's characterization ("Phase 3... carries no Ponytail rung") is
accurate. Confirmed by direct grep, not taken on the critic's word:

```
$ grep -n "Ponytail rung\|rung R[0-9]\|R[0-9] —\|resolved at.*R[0-9]" \
  reference/proposal-*.md docs/adr/*.md | grep -v product-workspace-split
docs/adr/0002-platform-boundary-shape.md:28: ...resolved at Ponytail rung R6...
reference/proposal-tui-t-series.md:61: Every normative decision names its lowest viable Ponytail rung...
reference/proposal-icm-r-procedure-authority.md: (§17 register, 11 rows)
```

No `R1`–`R7` token found anywhere in the SPLIT-1 proposal. Finding survives.

## Finding 2 — §4.7, §3.3, and Phase 5 ungraded against the ladder

**Verdict: SURVIVES, but weaker than presented and only for two of the three items.**

Tried to break this by noting the critic's own text concedes Phase 5 is "the
least concerning of the three" since it's workflow content, arguably R2 —
and further, the proposal's own §6 Phase 3 heading explicitly says "Only
new engine work in the plan," which is the proposal's own admission that
Phase 5 is *not* new engine work. That's a real weakening: bundling Phase 5
in with §4.7 and §3.3 overstates the case, since the proposal already
implicitly signals Phase 5's rung by calling Phase 3 the *only* new engine
work.

That said, §3.3 (scoped retrieval, new indexing architecture, no stated
floor at all) and §4.7 (edition marker, format explicitly left open at §11
item 2) are undischarged on their own terms — the contract's own axis
question ("is each minimum-sufficient, or reaching past R6?") cannot be
answered from proposal text for either, and no ADR 0014 ruling closes this
gap (checked: ADR 0014 decisions 1-13 don't state a rung for §3.3 or §4.7).
Finding survives for §3.3 and §4.7; the Phase-5 third of it is weak and
should be read as folded into Finding 1's Phase-3 concern rather than an
independent third data point.

## Finding 3 — §12 non-goals claims restraint without testing it where it bites

**Verdict: REFUTED — immaterial, parasitic on Findings 1 and 2.**

The finding is textually accurate (checked: none of Phase 3, §3.3, §4.7, or
Phase 5 appear in §12's non-goals list — grep confirms), and correctly notes
that a "what we won't build" list is not the same claim as "what we are
building sits at its lowest viable rung." But this is true by construction:
§12 was never the proposal's mechanism for demonstrating minimality — §13's
decision register and the phase descriptions are. The critic's own
conclusion says as much: "does not go as far as the axis needs it to,"
which is just Finding 1/2's gap restated through a different section. It
identifies no defect that fixing Findings 1 and 2 (adding rung citations
where §13/§6 describe the new work) would not already resolve. Per the
refutation ground "true but immaterial — would change nothing about what
happens next": nothing about §12 itself would need to change if Phase 3,
§3.3, and §4.7 were rung-tagged where they're actually described. Rated
"note" by the critic already, i.e. self-assessed as the weakest of the five;
I could not find independent content in it once 1 and 2 are accounted for.

## Finding 4 — `sgt init`'s embedded-template write path vs. ADR 0008

**Verdict: REFUTED — mislabels an open-decision-list gap as an invariant violation, and the ADR 0008 analogy is strained.**

Checked ADR 0008 directly. Its ruling governs `resolve_data_dir`
(`src/cli.rs:406-419`) — a function that walks up from `cwd` to find an
**already-existing** `sergeant.toml` for an **already-initialized** estate,
and decides where the daemon's own runtime state (`.sergeant/data`) lives
relative to that manifest. The "manifest should be authority for both or
neither" principle is scoped to that specific precedence-rung question
(`--data-dir` / `SGT_DATA_DIR` / estate discovery / XDG fallback), argued
from a concrete asymmetry (`surfaces_dir` is manifest-declarable,
`data_dir` was not, until this ADR closed the gap).

`sgt init` is a different operation in kind: it is the **estate-bootstrap**
step that creates the initial directory tree — the moment before any
`sergeant.toml` necessarily exists to be authoritative over anything. It is
also, as I confirmed by grep, not implemented anywhere in this codebase
today (`grep -n "\"init\"" src/cli.rs` — no match); it is proposed future
CLI surface gated behind the NORTH-STAR "prebuilt binary" amendment this
same proposal is drafting. Applying a ruling about an existing daemon's
data-dir *resolution order* to a not-yet-designed *bootstrap write* is an
analogy, not a citation — ADR 0008 does not "already settle the principle
for" a question it never had before it (bootstrap-time writes), and the
finding's own "Invariant violated" field overstates what is actually a
completeness gap: this belongs on the proposal's own §11 Open Decisions
list (which already tracks the structurally identical edition-marker-format
question) rather than being flagged as contradicting a settled rule. Silence
is not a violation unless the silence necessarily implies a contrary
decision, and nothing here does — verified no mention of `data_dir`,
`surfaces_dir`, `sergeant.toml`-as-authority, or `manifest` in the proposal
in the context of Phase 3's `sgt init` bullet (grep confirms `sergeant.toml`
appears only at proposal lines 138, 381, 516 — none of them Phase 3's
embedding bullet).

## Finding 5 — GAUNTLET.md and LESSONS.md never placed in either repository or substrate

**Verdict: SURVIVES — and I could not find a refutation.**

Tried three angles: (a) maybe the files are covered implicitly by a broader
category in §3.2's Evidence row; (b) maybe ADR 0014 already ruled on this
and the critic missed it; (c) maybe this is immaterial because the files are
small/legacy and wouldn't actually block the split.

All three fail:

(a) §3.2's Evidence row is an enumerated list — `docs/gauntlet/runs/`,
`resources/`, `reference-corpus/`, `reference/sergeant-upstream/` — not a
category with an implied "etc." Root-level `GAUNTLET.md` and `LESSONS.md`
are distinct paths from `docs/gauntlet/runs/` and are not textually covered.
Confirmed both are real, substantial root files, not stubs:

```
$ wc -l GAUNTLET.md LESSONS.md
  2983 GAUNTLET.md
   449 LESSONS.md
```

(b) Grepped ADR 0014 directly for both filenames: `GAUNTLET.md` does not
appear at all; `LESSONS.md` appears once, only as `LESSONS L12` inside a
citation, not as a ruling on where the file itself lives. No ruling exists
to have been missed.

(c) Immateriality fails on `docs/DEVELOPMENT.md`'s own text
(`docs/DEVELOPMENT.md:95-98`), which names exactly these two files under the
heading "The development record (read before changing method or scope)" —
the section a session is required to consult before changing repository
topology, which is precisely what this proposal does. `GAUNTLET.md` is also
where ADR 0014's own "Consequences" section (`docs/adr/0014...md`) and ADR
0008's Consequences section point for implementation follow-through — i.e.,
it is treated elsewhere in this repository's own governing documents as
load-bearing, not incidental.

Also checked: grepped the proposal for both strings. `GAUNTLET.md` and
`LESSONS.md` (with the `.md` suffix, i.e. as file references rather than as
citation shorthand) appear **zero times**; only "GAUNTLET rulings" (routing-
fog evidence row) and "LESSONS L20" (citation, twice) appear, neither in a
context that assigns the files a home. §3.1's product-repo list is
otherwise granular enough to name `SECURITY.md`, `CONTRIBUTING.md`, and
`CHANGELOG.md` individually, which makes the omission of two files an order
of magnitude larger than those look like a real gap rather than a stylistic
elision.

## Summary

| # | Critic verdict | Refuter verdict |
|---|---|---|
| 1 | error | **SURVIVES** — confirmed by grep against this repo's own precedent (T-SERIES, ICM-R proposals both carry pre-implementation rung registers; SPLIT-1 has zero R1–R7 tokens) |
| 2 | warning | **SURVIVES** for §3.3/§4.7; Phase-5 third of the claim is weak (proposal's own "only new engine work" framing implicitly clears Phase 5) |
| 3 | note | **REFUTED** — immaterial, fully parasitic on Findings 1–2 |
| 4 | warning | **REFUTED** — conflates a bootstrap-time write (`sgt init`, unimplemented) with ADR 0008's scope (an existing daemon's data-dir resolution order); properly a §11-class open decision, not an invariant violation |
| 5 | warning | **SURVIVES** — verified directly: zero file-reference mentions of `GAUNTLET.md`/`LESSONS.md` in the proposal, no ADR 0014 ruling on their placement, both are large (2,983 / 449 line) files `docs/DEVELOPMENT.md` itself names as required reading before exactly this kind of change |

Three of five findings survive independent refutation attempts (1, 2, 5);
two do not (3, 4). No finding here invalidates the proposal's architecture
outright, consistent with the critic's own summary — but Findings 1 and 5 in
particular identify checkable, reproducible gaps (absent rung citations
against this project's own proposal-writing convention; two large,
by-name-required development-record files with no assigned home) rather
than stylistic preferences.
