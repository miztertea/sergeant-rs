# SPLIT-1 — critic: assumptions

Blind critic, seat 3 of 4, axis = assumptions. Grades
`reference/proposal-product-workspace-split.md` §1–§13 per
`docs/gauntlet/contracts/SPLIT-1.md`'s assumptions axis. Empirical: every
count below was reproduced by direct command, not read off the proposal.
Working directory for all commands: repo root.

## Counts: claimed vs measured

| Count | Claimed | Measured | Status |
|---|---|---|---|
| Template `.md` files under `.sergeant/workflows` + `skills` | 216 | 216 | **holds** |
| Contamination candidates | 162 | not independently reproducible; my best reconstruction lands ≈117 (see Finding 1) | **does not hold as stated** |
| Hard references into `reference/sergeant-upstream/` | 91 | 91 files | **holds** (file count; occurrence count is 341 — see Finding 8) |
| `BU-####` citations | 611 across 89 files | 611 occurrences, 89 files | **holds exactly** |
| Stage `CONTEXT.md` files | 80 | 80 (files at `.sergeant/workflows/*/[0-9]*/CONTEXT.md`) | **holds** |
| Blocks titled "Bounded judgment" | 82 | 82 (across all 216 template `.md` files, any heading level, case-insensitive) | **holds**, but only when the search widens past the 80 stage `CONTEXT.md` files to the full 216-file corpus — the two "80" and "82" numbers are drawn from different scopes and the proposal doesn't say so |
| Files citing `@@bounded-judgment` | 77 | 79 | **off by 2** |
| `@@bounded-judgment` occurrences | 107 | 109 | **off by 2** |
| `@@tdd` occurrences | 10 | 14 | **off by 4** |
| `@@name` occurrences | 6 | 8 | **off by 2** |
| `@@test-quality`, `@@domain-modeling`, `@@ticket-shaping`, `@@triage-state-machine` | 4, 1, 1, 1 | 4, 1, 1, 1 | **hold exactly** |
| Unresolved `@@refs` | 3 | 3 (`@@domain-modeling`, `@@ticket-shaping`, `@@triage-state-machine`, all explicitly named as gaps in `wayfinder/CONTEXT.md` lines 69–75) | **holds** |

Commands used for the headline totals:

```
find .sergeant/workflows skills -type f -name "*.md" | wc -l                     # 216
grep -rl "reference/sergeant-upstream" .sergeant/workflows skills --include="*.md" | wc -l   # 91
grep -ro "BU-[0-9P][0-9A-Za-z-]*" .sergeant/workflows skills --include="*.md" | wc -l         # 611
grep -rl "BU-[0-9P][0-9A-Za-z-]*" .sergeant/workflows skills --include="*.md" | wc -l         # 89
find .sergeant/workflows -path "*/[0-9]*/CONTEXT.md" | wc -l                                  # 80
grep -rE "^#+ Bounded [Jj]udgment" .sergeant/workflows skills --include="*.md" | wc -l         # 82
grep -rl "@@bounded-judgment" .sergeant/workflows skills --include="*.md" | wc -l              # 79
grep -roE "@@[a-z-]+" .sergeant/workflows skills --include="*.md" | sort | uniq -c             # per-ref breakdown
```

---

## Findings

### 1. [error] The 162 "contamination candidates" figure has no reproducible methodology, and my best reconstruction falls well short of it

> "template .md files under .sergeant/workflows + skills 216 / carrying
> host/repo/tool-specific references (candidates) 162"

The proposal states no grep pattern for this count (unlike the "unambiguous"
91 and 611 counts, which are self-evidently `reference/sergeant-upstream`
and `BU-####` matches). I reconstructed it two ways:

- **Union of the two named "unambiguous" sets** (files citing `BU-####` ∪
  files referencing `reference/sergeant-upstream/`): 89 ∪ 91 = **100 files**
  (9 BU-only, 11 upstream-only, 80 in both).
- **Broadest plausible "host/repo/tool-specific" grep** over the remaining
  ~117 files not already in that union — `cargo|rustc|\.rs\b|Cargo\.toml|
  sergeant-rs\b|\.sergeant/|src/|clippy|GAUNTLET|docs/adr|AGENTS\.md|
  NORTH-STAR|gh pr|/home/|miztertea` — found only **17 additional files**.

That totals ≈117, not 162 — a gap of 45 files (≈28% of the claimed figure)
that no combination of reasonable patterns I tried closes. Either the
author used a materially different (and unstated) definition of
"host/repo/tool-specific," or the 162 figure is itself inflated. Given
§2.2 leans on 162 as the headline "measured" number and explicitly uses it
to argue Phase 2 is a necessary prerequisite, an unreproducible input to
that argument is a defect in the measurement, not just a rounding error.

### 2. [warning] Sampling the residual corpus suggests real triage-needing contamination is closer to the "unambiguous" 100 than to 162

Per the contract's instruction to sample at least 15 files and classify
genuine contamination vs. legitimate content: I drew a random sample of 20
files from the full 216-file corpus (`shuf --random-source=<(yes 42) -n
20`), then separately inspected 15 files drawn from the ~117 files *outside*
the unambiguous BU/upstream union (`shuf --random-source=<(yes 7) -n 15`).

Findings from the second sample: 6 of 15 (`output/README.md` stubs under
`deepen-module`, `diagnose-bug`, `cross-repo-work`, `dispatch`) are 9-line
boilerplate with **zero** host/repo/tool-specific content of any kind — they
are generic output-shape placeholders. The remainder that did match were
almost entirely self-referential paths to the workflow's own shipped
machinery (e.g. `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py`,
`../scripts/finalize.py`) — paths that will exist in every installation of
the shipped product, not stale references to something that "won't ship."
Calling these "contamination" is a stretch; they're the templates
correctly referencing their own sibling files.

This matters for §2.2's argument: if the genuinely ambiguous residual
(neither unambiguous-BU/upstream nor self-referential-to-own-package) is
small — my sample suggests well under half of the disputed 62-file gap
between 100 and 162 — then the proposal's framing ("162 template files
carry host/repo/tool-specific references, requiring triage") overstates
how much of the 216-file corpus is actually in question. The unambiguous
100 (already conceded as needing Phase 2 work) may be doing all the real
argumentative work; the extra ~62 candidates add rhetorical weight without
much additional substance. This weakens, but does not eliminate, Phase 2's
position as a prerequisite — the unambiguous 100/216 (46%) alone is still
a real number.

### 3. [error] The "Routing fog" RED score's central evidence claim does not hold, and the true state of the evidence contradicts it

> "Routing fog | RED | ... The bounded-judgment ladder declares itself
> canonical in three places at once."

I searched every file that references `bounded-judgment.md` for
self-declared canonicity. Only **one** self-declaration exists — the source
file itself: `.sergeant/common/contexts/bounded-judgment.md:3`, "Canonical
source. Ratified by `docs/adr/0013-icm-r0-owner-rulings.md` decision 1...
no package copies this text; each adds only its local specialization."
Every other file I found that discusses the ladder's canonicity does the
opposite of competing for the title — `docs/icm/convention.md:406` reads:
"Two ladders, canonical sources fixed elsewhere, referenced here rather
than [duplicated]." That is a document explicitly *deferring* to a single
canonical source, which is the textbook fix for routing fog, not an
instance of it.

I could not find a second or third place declaring the ladder canonical.
This is the single counter-example the contract asks for: the score's
cited failure mode (multiple competing canonical claims) is not present in
the artifact; if anything the artifact shows the opposite (one declared
source, disciplined deferral elsewhere). This reads as motivated reasoning
toward a RED verdict the section had already committed to (six for six).
The rest of "Routing fog"'s evidence (four homes for "why did we choose
X") may still be defensible on its own; the specific "three places at
once" clause is not, and should not survive as written.

### 4. [warning] The "Dark corner" RED score overclaims against the repository's own LESSONS.md and GAUNTLET.md

> "Dark corner | RED | No durable home for episodic session memory or for
> rejected approaches. Decisions reached in conversation have nowhere to
> land until someone writes a proposal."

`LESSONS.md`'s own header (lines 3–7) states its purpose directly:
"Memory file across gauntlet units. One lesson per entry... Corrections
and confirmed approaches alike, with why they mattered." `GAUNTLET.md` is
an append-only ledger of run outcomes. Both are durable, both predate this
proposal, and both are explicitly designed to hold exactly the class of
content the RED score says has no home — including corrections (a
proxy for rejected approaches) and confirmed approaches (episodic memory
of what worked). L20 itself (cited elsewhere in this same proposal, §1) is
a durable record of exactly the kind the score says doesn't exist.

The narrower reading — "conversational decisions *before* anyone writes
anything down* have nowhere to land" — is defensible and probably what was
intended. As literally written, the score doesn't survive the
counter-example of the file the proposal itself cites two paragraphs
earlier in the same document.

### 5. [warning] The LESSONS L20 citation in §1 overstates what the record documents

> "a session hand-rolled seven stages of validate-and-ship because the
> real procedure 'never reached the catalog' (LESSONS L20)"

`LESSONS.md:146–185` (L20, "Stale-but-true is the state that never
triggers its own supersession") says the session drove `no-mistakes` by
hand for a day, "reimplementing `validate-and-ship`'s `40-drive-gates` and
`50-reconcile-custody` badly" — naming **two** stages, not seven. It is
true that `validate-and-ship` has exactly seven numbered stages
(`00`–`60`, verified: `00-check-scope, 10-do-the-work,
20-select-intent-transport, 30-start-run, 40-drive-gates,
50-reconcile-custody, 60-close-out`), so "seven stages" is defensible as
shorthand for "the whole workflow" rather than a literal stage-by-stage
reimplementation count. But L20's own text supports only two stages as
actually reimplemented; the proposal's phrasing borrows a precision the
source entry doesn't provide. Minor, but it is exactly the kind of
citation-inflation the assumptions axis exists to catch.

### 6. [note] The other two arrival-gap incidents check out exactly

- **`pkill` double-kill** (§1, §2.1): `docs/DEVELOPMENT.md:52` reads "A
  2026-08-14 session quoted this rule in a dispatch brief and then killed
  its own shell with an unbracketed `pkill` twice" — matches the proposal's
  claim verbatim in substance.
- **CI proposal's missed NORTH-STAR gate** (§1, §2.1): `grep -c
  "NORTH-STAR" ~/inbox/proposal-ci-cd-release-engineering.md` returns
  **0**; likewise 0 hits for "prebuilt" or "stranger"/"onboarding," despite
  the document reconciling carefully against ADR 0001, ADR 0004, and the
  S-series (all present, cited by section/line). The proposal's claim that
  it "reconciled carefully... while missing that NORTH-STAR holds its
  central deliverable behind a gate" is accurate.

### 7. [note] The `@@ref` breakdown table in §2.3 doesn't reproduce exactly, though the headline totals do

Measured vs. claimed: `@@bounded-judgment` 109 vs. 107, `@@tdd` 14 vs. 10,
`@@name` 8 vs. 6 (all other refs match exactly: test-quality 4,
domain-modeling 1, ticket-shaping 1, triage-state-machine 1). File-count
for "citing `@@bounded-judgment`" measures 79 against a claimed 77. These
are small, directionally-consistent discrepancies (measured always ≥
claimed), consistent with a slightly different exclusion rule (e.g.
excluding a handful of occurrences inside code fences or `references/`
subdirectories) that the proposal doesn't state. Not large enough to
affect §2.3's conclusion (three unresolved refs, which holds exactly), but
worth recording: none of the sub-counts in this table are mechanically
reproducible from the proposal text as written.

### 8. [note] The procedure/state test does not cleanly partition all 216 files

§2.2 and §4.7 frame the corpus as splitting into "procedure" (decays on a
human timescale, ships) and "state" (ownership/layout/architecture facts,
does not ship). In practice a large share of the stage `CONTEXT.md` files
carry a third category: citation/provenance apparatus — `BU-####` ids
paired with `reference/sergeant-upstream/...` locators — that is neither
the procedural instruction itself nor a state-fact about an installation.
It's evidentiary backing *for* a procedure, tied to a corpus that won't
ship. Phase 2's own remedy — "inline the substance, or drop the citation"
(§6) — is a third disposition rule, distinct from "ships as procedure" or
"doesn't ship because it's state." That the plan needs a third rule to
handle this material is itself evidence the binary partition doesn't fully
cover the 216 files, even though it's a reasonable first cut for the
majority of them.

### 9. [note] "91 hard references" is ambiguous between file-count and occurrence-count, but resolves correctly

`grep -rl` (files) gives exactly 91; `grep -ro` (occurrences) gives 341.
The proposal's number matches the file count exactly, so the ambiguity
resolves in the proposal's favor — flagged only so a future reader knows
which reading was verified.

---

## Summary

Of the eight explicitly-named counts in the contract, six reproduce
exactly (216, 91, 611/89, 80, three unresolved refs, and the three
smallest `@@ref` sub-counts) and two do not (162 candidates; the
`@@bounded-judgment`/`@@tdd`/`@@name` sub-breakdown, off by small amounts).
The 162 figure is the more consequential failure: it's unreproducible as
stated, and independent sampling suggests the real triage-needing set is
smaller than claimed, which weakens (without eliminating) Phase 2's
position as a hard prerequisite. Two of six "Six Memory Failure Modes" RED
scores (Routing fog, Dark corner) rest on evidence that doesn't survive a
direct counter-example from this same repository's own files. The three
arrival-gap incidents mostly check out, with one (L20 "seven stages")
mildly overstated relative to its source.
