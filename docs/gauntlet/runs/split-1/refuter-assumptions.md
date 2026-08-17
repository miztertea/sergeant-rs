# SPLIT-1 — refuter: assumptions

Refuter seat, axis = assumptions. Fresh context, instructed to refute
`docs/gauntlet/runs/split-1/critic-assumptions.md`'s nine findings against
`reference/proposal-product-workspace-split.md`, defaulting to REFUTED when
uncertain, per `docs/gauntlet/contracts/SPLIT-1.md`. I did not read the
other three critic files or any other refuter's output. Every measurement
below was re-run independently against the working tree, not copied from
the critic's report.

Working directory for all commands: repo root.

---

## Finding 1 — 162 "contamination candidates" has no reproducible methodology

**Verdict: SURVIVES**

Independently reran the critic's reconstruction:

```
grep -rl "reference/sergeant-upstream" .sergeant/workflows skills --include="*.md" > upstream.txt   # 91
grep -rl "BU-[0-9P][0-9A-Za-z-]*" .sergeant/workflows skills --include="*.md" > bu.txt               # 89
cat upstream.txt bu.txt | sort -u | wc -l    # 100 (union)
216 - 100 = 116-117 files outside the union
```

I confirm the union is exactly 100 and the residual outside it is ~117
files. I also checked whether ADR 0014 — the fidelity authority — cites
162 anywhere: it does not. `grep -n "162" docs/adr/0014-*.md` returns
nothing. The ADR's own "Consequences" section states only the two
unambiguous counts ("91 template files reference
`reference/sergeant-upstream/`... and 611 `BU-####` citations"), skipping
162 entirely even though it would be the natural place to carry the
number forward if it had been validated in the owner session.

Attempted refutation angles and why they fail:
- *The proposal already hedges 162 as "an upper bound requiring
  triage — some matches are legitimate examples in a Rust context."*
  True, but a hedge about precision is not the same as a hedge about
  reproducibility. The critic isn't complaining that 162 double-counts
  ambiguous matches; the complaint is that no stated grep pattern —
  reasonable or generous — reaches 162 at all. That's a different and
  more serious problem than the proposal's own caveat covers.
- *Maybe it's immaterial — Phase 2 triages either way, so the exact
  number doesn't change what happens next.* This is contradicted by
  §2.2's own text, which uses 162 as part of the case that Phase 2 is a
  hard prerequisite ("moving the record without decontaminating the
  templates dangles references"). If the true count is materially
  smaller, that argument gets weaker, which is exactly what Finding 2
  goes on to show. Not immaterial.

I could not find a grep pattern, disclosed or reconstructed, that
produces 162. This finding stands.

## Finding 2 — sampling suggests real contamination is closer to 100 than 162

**Verdict: SURVIVES**

Spot-checked the critic's characterization rather than its exact random
sample (different seed, not independently reproducible bit-for-bit).
Pulled a stub file from the residual set directly:

```
$ wc -l .sergeant/workflows/deepen-module/00-classify-dependencies/output/README.md
9 .sergeant/workflows/deepen-module/00-classify-dependencies/output/README.md
```

Contents are generic `output/README.md` boilerplate — a Layer-4 disposition
note with zero host/repo/tool-specific content, structurally identical
across every stage directory in the corpus. This corroborates the critic's
claim that a chunk of the "candidates" residual is generic scaffolding, not
contamination.

Attempted refutation: the finding is explicitly hedged ("weakens, but does
not eliminate... the unambiguous 100/216 (46%) alone is still a real
number") — it does not overclaim that Phase 2 is unnecessary, only that its
scope may be smaller than stated. I could not find grounds to call this
motivated or unsupported; my own spot check points the same direction.
Survives, and inherits Finding 1's materiality.

## Finding 3 — "Routing fog" RED score's "three places at once" claim

**Verdict: SURVIVES**

Checked every file that discusses the bounded-judgment ladder's canonicity:

```
$ grep -n -i canonical .sergeant/common/contexts/bounded-judgment.md
3:Canonical source. Ratified by docs/adr/0013-icm-r0-owner-rulings.md
83:Canonical shape:
```

Only one self-declaration exists. `docs/icm/convention.md:405-407` reads
"Two ladders, canonical sources fixed elsewhere, referenced here rather
than restated" — explicit deference, not a competing claim. I searched
`GAUNTLET.md` and `.sergeant/index.md` for "canonical" near the ladder
(both files use "canonical" elsewhere, for unrelated things — a Work
surface, a skill root) and searched broadly for "authoritative" /
"definitive" / "source of truth" near bounded-judgment content — nothing
surfaced a second or third self-declared canonical claim for the ladder
specifically.

Attempted refutation: maybe "three places at once" refers to the broader
"why did we choose X has four homes" claim in the same table cell, and the
critic is being uncharitable by isolating the ladder clause. But the two
clauses are grammatically distinct sentences in the proposal's own text —
the four-homes claim and the "declares itself canonical in three places"
claim are separate assertions, and only the first is defensible. I cannot
construct a reading of the artifact where the second is true. Survives,
but note for the adjudicator: this narrows one clause of one RED score's
evidence, not the RED verdict itself — the four-homes claim is untouched
and independently sufficient to keep Routing fog contestable.

## Finding 4 — "Dark corner" RED score vs. LESSONS.md and GAUNTLET.md

**Verdict: SURVIVES**

```
$ sed -n '1,7p' LESSONS.md
Memory file across gauntlet units. One lesson per entry, one-line summary
first. Corrections and confirmed approaches alike, with why they mattered.
```

`GAUNTLET.md` opens as "Development record... append-only: entries record
what happened, with evidence; superseded decisions stay visible." Both
predate the proposal and both hold exactly the class of content ("no
durable home for episodic session memory or for rejected approaches")
the RED score says is absent.

Attempted refutation: the proposal's diagnostic might mean something
narrower — decisions still *in flight*, before anyone writes a lesson or
ledger entry, genuinely have nowhere to land until someone acts. That
narrower reading is real and the critic concedes it directly ("probably
what was intended"). But the score as literally written in the table cell
doesn't qualify itself that way, and the critic's counter-example (L20,
cited two paragraphs earlier in the same proposal) is a durable record of
exactly the failure the cell claims doesn't exist. As written, survives.

## Finding 5 — L20 "seven stages" overstates the source

**Verdict: SURVIVES (minor)**

```
$ sed -n '146,152p' LESSONS.md
...reimplementing validate-and-ship's `40-drive-gates` and
`50-reconcile-custody` badly...
```

Only two stages are named in L20. Confirmed the workflow itself has seven
numbered stages:

```
$ ls .sergeant/workflows/validate-and-ship/ | grep -E '^[0-9]'
00-check-scope 10-do-the-work 20-select-intent-transport 30-start-run
40-drive-gates 50-reconcile-custody 60-close-out
```

Attempted refutation: "seven stages" is a defensible gloss for "hand-rolled
the whole workflow," and the workflow does have seven stages — so the
number isn't fabricated, it's borrowed from the wrong referent (workflow
stage-count instead of stages-actually-reimplemented). That's a real
distinction but a narrow one. The critic already rates this severity
correctly (mildly overstated, not a defect that changes any argument) and
does not lean on it for anything downstream. Survives as a minor citation
imprecision, immaterial to any of the proposal's decisions.

## Finding 6 — pkill and CI-proposal incidents check out

**Verdict: SURVIVES (non-adversarial — confirms the proposal, not a defect)**

```
$ grep -c "NORTH-STAR" ~/inbox/proposal-ci-cd-release-engineering.md
0
$ grep -ci "prebuilt" ~/inbox/proposal-ci-cd-release-engineering.md
0
$ grep -ci "stranger" ~/inbox/proposal-ci-cd-release-engineering.md
0
```

`docs/DEVELOPMENT.md`'s pkill passage matches the proposal's claim in
substance exactly, including the "twice" detail. Both source citations
check out. I flag this because it is not really a "finding" against the
proposal — it validates accuracy rather than identifying a defect — so it
should not be counted toward anything that would push the unit toward
"sent back." Recorded as surviving only in the sense that the critic's
own claim (the sources say what the proposal says they say) is itself
correct.

## Finding 7 — `@@ref` breakdown table doesn't reproduce exactly

**Verdict: SURVIVES**

```
$ grep -roE "@@[a-z-]+" .sergeant/workflows skills --include="*.md" | sort | uniq -c
    109 @@bounded-judgment
      1 @@domain-modeling
      8 @@name
     14 @@tdd
      4 @@test-quality
      1 @@ticket-shaping
      1 @@triage-state-machine
```

Matches the critic's numbers exactly: 109 vs. claimed 107, 14 vs. 10, 8
vs. 6, and the three smallest sub-counts (4, 1, 1, 1) hold exactly, as
does the headline "three unresolved `@@refs`." File-count for "citing
`@@bounded-judgment`" is 79 against a claimed 77, also confirmed.

Attempted refutation: could a different but equally reasonable exclusion
rule (code fences, `references/` subdirectory) produce exactly the
proposal's numbers, making both readings valid? I did not test every
possible exclusion rule, but the critic's point stands regardless of which
rule is "more correct": the proposal states these sub-counts as measured
without disclosing a method, and a plain grep — the obvious first attempt
any reader would make — does not reproduce them. That gap is the finding,
not which side is right. Survives, materiality is low (the three
smallest counts and the "three unresolved" headline, which is what §2.3
actually leans on, all hold).

## Finding 8 — procedure/state test doesn't cleanly partition all 216 files

**Verdict: SURVIVES (low materiality)**

Confirmed the cited text exists as quoted:

```
$ grep -n "inline the substance, or drop the citation" reference/proposal-product-workspace-split.md
364:  inline the substance, or drop the citation. A shipped template may not
```

Attempted refutation: a two-bucket model can still be "correct" if the
third disposition rule in §6 is read as an operational instruction for
handling a sub-case of "state," not evidence of a genuine third category.
That's a plausible reading, but the critic's point — that citation/
provenance apparatus (a `BU-####` id plus an upstream locator) is neither
a procedural instruction nor a fact about an installation — is a fair
description of what that content actually is. The critic explicitly
concedes this is "a reasonable first cut for the majority" of files, which
keeps the finding narrow and honest rather than overclaiming the whole
partition is broken. Survives as a genuine, low-stakes observation.

## Finding 9 — "91 hard references" file-count vs. occurrence-count ambiguity

**Verdict: SURVIVES, but flagged as non-adversarial**

```
$ grep -rl "reference/sergeant-upstream" .sergeant/workflows skills --include="*.md" | wc -l
91
$ grep -ro "reference/sergeant-upstream" .sergeant/workflows skills --include="*.md" | wc -l
341
```

Both numbers confirmed. The critic explicitly states this "resolves in
the proposal's favor" — i.e., the proposal's 91 is correct under the file-
count reading and the ambiguity is disclosed only so a future reader knows
which reading was checked. This is not a defect in the proposal; it is a
methodology note the critic recorded honestly rather than inflating into
a finding against the artifact. Nothing to refute because nothing is
being claimed as wrong.

---

## Summary for adjudication

Of the nine findings, none were breakable by independent re-measurement —
every count I reran matched what the critic reported, and every source
citation I checked said what the critic said it said (or didn't say what
the proposal claimed, in the two citation-inflation cases). This is not
the outcome I expected going in as an adversarial seat; I looked
specifically for grep-pattern differences, alternate readings of "canonical,"
and narrower interpretations of the RED-score cells that would let the
proposal's framing stand, and none of them closed the gaps.

That said, materiality varies sharply:

- **Findings 1 and 2 are the load-bearing pair.** The 162 figure is
  unreproducible by any method I or the critic tried, tops out around
  117 by the most generous plausible grep, and is absent from ADR 0014's
  own consequences section even though 91 and 611 are both there. This
  bears on whether §2.2's framing of Phase 2 as a hard prerequisite is
  itself overstated — worth the adjudicator's attention, not just a
  measurement footnote.
- **Finding 3** narrows one clause in one cell of "Six for six" — real,
  but the cell's other evidence (four homes for "why did we choose X") is
  untouched, so it does not on its own break the RED score.
- **Finding 4** has the same shape: a real overclaim in the score's
  literal text, with a narrower reading available that the critic itself
  names as "probably what was intended."
- **Findings 5, 7, 8, 9 are minor-to-immaterial** — citation imprecision
  or disclosed ambiguity that doesn't change any downstream argument.
- **Finding 6 is not adversarial** — it confirms the proposal is accurate
  and should not count as a mark against it.

One meta-point for the adjudicator, per the contract's own instruction to
note where a finding only holds because the contract told the critic to
look for it: the contract's assumptions axis explicitly directs critics to
"reproduce the counts... run the greps... report any that do not hold."
Findings 1, 2, 7, and 9 are direct executions of that instruction, not
independent discoveries — which doesn't make them wrong (I verified all of
them personally), but it does mean the contract, written by the artifact's
own author, pre-selected exactly the kind of scrutiny that would catch
this specific class of error (unstated grep methodology) while not
equally directing scrutiny toward, say, whether the six RED scores as a
*set* are well-calibrated against a non-RED baseline. The critic went
beyond the contract's script on Findings 3–6 and 8, which is where the
more interesting (if lower-materiality) findings live.
