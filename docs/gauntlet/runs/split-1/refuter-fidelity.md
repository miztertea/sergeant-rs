# SPLIT-1 — Refuter: fidelity

Refuter seat, axis **fidelity**. Target: `docs/gauntlet/runs/split-1/critic-fidelity.md`.
Instructed to refute, defaulting to refuted when uncertain. Did not read
critic-invariants.md, critic-assumptions.md, critic-sequencing.md, or any
other refuter's output (one incidental filename, `critic-sequencing.md`,
surfaced in a directory grep of unrelated keywords — its content was not
opened or used).

Method: read `docs/gauntlet/contracts/SPLIT-1.md`, the full proposal
(`reference/proposal-product-workspace-split.md`, 558 lines), and the full
ADR 0014 (190 lines) independently. Re-ran every grep the critic cites and
several the critic didn't run, to check the critic's own sourcing, not just
the proposal's.

One standing observation before the findings: the SPLIT-1 contract's
fidelity section — including the four-corrections checklist the critic
built its method around — was written by Captain, the same author as the
proposal under review. Where a finding's *only* basis is "the contract
named this and the proposal doesn't say it," that is Captain grading
Captain by a rubric Captain wrote, and I've flagged it inline rather than
either dismissing it (the contract is still the operative grading document
for this unit) or treating it as neutral.

---

### Finding 1 — Notion correction absent

**Verdict: SURVIVES.**

Confirmed independently: `grep -in notion reference/proposal-product-workspace-split.md`
returns nothing, and no synonym (`idea store`, `personal`, `IdeaOS`) appears
either — so this isn't a case of the critic missing a rephrased mention.
ADR 0014 decision/alternative section (lines 150–155) does record the
correction in the owner's own words, so the critic's cited source says what
it claims.

I tried to break this two ways and couldn't:

- *Is the proposal's silence excused because ADR 0014 already carries the
  history and the proposal isn't obligated to duplicate it?* No — the
  contract's fidelity axis makes this an explicit, named check ("the
  proposal must not read as though this was never entertained"), not an
  inference from the general "trace to ADR 0014" standard. The contract is
  the operative document for this unit regardless of who wrote it.
- *Is it immaterial — would nothing change if it stayed missing?* No. §3.2–
  §3.3 design the exact knowledge-layer alternative that Notion was
  rejected in favor of, in the "obvious and only answer" voice the axis
  specifically warns about. A future reader with no access to this ADR (or
  to a conversation that "exists nowhere else") would have no way to
  recover that Notion was ever on the table. That's the axis's stated
  failure mode, not a technicality.

Flag: this finding's existence traces entirely to the contract's named
checklist, which Captain also authored — noted per the standing
observation above. It still survives because the underlying fact (total
silence on a documented, named correction) is independently verifiable and
the standard, however self-imposed, is explicit in the grading document.

---

### Finding 2 — three-repository / dependency-inversion correction absent

**Verdict: SURVIVES.**

Confirmed: `grep -in "three repositor"` and `grep -in invert` against the
proposal both return nothing. ADR 0014 decision 5 (lines 77–80) records
Captain's original three-repo/extraction proposal and the owner's
correction to two repos with the dependency reversed (estate consumes the
*released* distro, not a build-tree copy). I additionally checked whether
the proposal at least captures the *consequence* of that correction even
without narrating the history — it doesn't: `grep -in "bootstrap\|working
tree\|build-tree"` against the proposal is empty, so even ADR 0014's own
named consequence of this ruling (the "bootstrap hazard" — a stock-template
change can't be exercised by the estate until released, and pointing the
estate at a working tree is "a legitimate testing posture and a corrosive
default") is absent from the proposal too. That's a second, independent
gap beyond what the critic flagged, not a refutation of it.

Same contract-provenance flag as Finding 1 applies and doesn't change the
verdict for the same reason.

---

### Finding 3 — Gmail/inbox-convention correction absent

**Verdict: REFUTED.**

The critic's finding rests on the claim that this is one of "four recorded
corrections" and that "ADR 0014 records all four" (SPLIT-1.md line ~90).
I checked this directly:

```
grep -rin "gmail" --include="*.md" . | grep -v /.git/
```

Only three hits exist in the whole repository: the contract itself
(`docs/gauntlet/contracts/SPLIT-1.md`), and the critic's own file
(`docs/gauntlet/runs/split-1/critic-fidelity.md`). **ADR 0014 — the
document the contract itself names as "the fidelity authority" and the
document this whole axis is supposed to grade the proposal against — never
mentions Gmail, cerberus.md, or an inbox-retrieval failure by Captain, at
all.** ADR 0014's context section states only that the inbox proposal "was
surfaced by the owner mid-session via the inbox convention
(`docs/environments/cerberus.md`)" — a neutral statement of how the
document arrived, with no mention of Captain reaching for Gmail or failing
to read the convention first.

This breaks the finding on its own terms: the critic says the proposal
should trace to ADR 0014 and doesn't; but the correction it's demanding
doesn't trace to ADR 0014 either — it traces only to the contract's own
uncorroborated narration, written by the same author as the artifact under
review. Per the contract's own artifact section, ADR 0014 is "the fidelity
authority" and the standard is "every decision in the proposal must trace
here or to cited evidence" — that cuts both ways. A correction that isn't
itself in the fidelity authority isn't a fact the proposal can be faulted
for omitting by that authority's own standard.

This is also the clearest instance of the standing observation: this
finding exists *only* because the contract told the critic to look for it,
and the contract's claim about what "ADR 0014 records" is, on inspection,
false for this item. Grounds for refutation: cites a record that does not
say what it claims.

---

### Finding 4 — `workflow diff` correction survives in outcome but not narrative

**Verdict: SURVIVES, weakly.**

Confirmed: `grep -in "workflow diff"` against the proposal hits only two
places (§4.7 line 307, §12 line 529), both stating the final position (no
verb, edition marker instead) with no narration that Captain first proposed
the verb, withdrew it under R1, and then had to be corrected back toward
restoring the underlying property. ADR 0014 decision 4 does record that
two-step shape. I tried to break this by checking whether §4.7's phrasing
of "a fork has no invalidation mechanism" is close enough to ADR 0014's
own framing that it implicitly signals a prior-and-reversed position — it
isn't: the sentence reads as a single derivation ("no invalidation
mechanism → therefore edition marker"), not as a corrected one. The critic
correctly self-rates this the mildest of the four (a "warning," not an
"error") because the *outcome* is faithfully stated — I agree with that
calibration and don't find grounds to lower it further to refuted, since
the contract explicitly asks whether the correction's *shape* survives, not
just its outcome.

---

### Finding 5 — PACE and succession-of-authority: invented scope

**Verdict: SURVIVES — this is the strongest finding in the file.**

This is the one finding that hits the fidelity axis's actual stated failure
mode ("invented scope") directly, independent of the four-corrections
checklist, so the contract-provenance flag doesn't apply here.

Checked directly:

```
grep -in "pace" docs/adr/0014-product-workspace-split-owner-rulings.md   → no hits
grep -in "succession" docs/adr/0014-product-workspace-split-owner-rulings.md → no hits
```

ADR 0014's thirteen decisions cover distro delivery, versioning, templates,
the `workflow diff` withdrawal, repository topology, workspace history,
`DEVELOPMENT.md` placement, NORTH-STAR amendments, the inbox re-scoping,
both existing ladders shipping inline (decision 10 — explicitly J5–J0 and
Ponytail R1–R7 only), knowledge-organization priority, anti-capture
balance, and model policy. None authorizes new rungs below J0. O-SMEAC
appears once, in the context section's list of corpus items the owner had
Captain read — not as a ruling, and not connected in that section (or
anywhere else in the ADR) to PACE or succession-of-authority by name. I
also checked the rest of the repo for any independent authorization:

```
grep -rin "succession" --include="*.md" docs/ NORTH-STAR.md .sergeant/common/  → no hits
grep -rin "\bpace\b" --include="*.md" docs/ NORTH-STAR.md .sergeant/common/   → no relevant hits (only unrelated uses of "pace" as in "at their own pace")
```

No ruling anywhere authorizes this addition. §4.4 introduces it as settled
doctrine, §9 builds live overnight-authority machinery on top of it (the
PACE Primary/Alternate/Contingency/Emergency framing at lines 479–483), and
§13's decision register records "PACE and succession added below J0" in
the same unqualified voice as items that do trace to an explicit owner
ruling (e.g., "Authority ladder | J5–J0 retained" in the same cell, which
*is* decision 10). Contrast with §1's relationship table, which correctly
flags the NORTH-STAR gated item as "requires a dated owner ruling" — §4.4
gets no equivalent flag despite having strictly less backing. I could not
find a way to break this: it is exactly the invented-scope failure mode the
axis exists to catch, it's material (§9 grants J4/J2 unattended authority
partly structured around these two new rungs), and it doesn't depend on the
contract's four-corrections checklist at all.

---

### Finding 6 — unattributed blockquote/"verbatim" claims in §4

**Verdict: REFUTED.**

The finding bundles two claims; both break under inspection.

First, the §4.2 blockquote ("A permission system alone cannot produce
judgment...") is treated as an unattributed *external* quote requiring a
named source. I checked the document's own use of blockquote formatting:

```
grep -n "^>" reference/proposal-product-workspace-split.md
```

The only other blockquote in the document is §5's "Amended North Star
statement" (lines 314–323) — demonstrably original prose written for this
proposal (it's the proposed *new* NORTH-STAR text, not a citation of
anything). That establishes the document's own house style: blockquote
formatting is used here for emphasis/pull-quote effect, not as a citation
marker. Reading the §4.2 blockquote as an implied claim of external
verbatim sourcing is importing a convention the document doesn't use
anywhere else in it. That breaks the strongest part of this finding.

Second, §4.5's "Adopted verbatim" phrase for the failure-attribution
taxonomy is a real gap — no source is named — but the critic's own
severity rating (`[note]`, the lowest of the three tiers used in the
report) and own hedge ("probably legitimate... the source material
plausibly exists among the inputs the owner had Captain read") concede
this doesn't rise to a confirmed fidelity break. Combined with the broken
blockquote half of the finding, and the contract's "would change nothing
about what happens next" standard (a five-row attribution taxonomy with a
plausible-but-unnamed source doesn't invalidate any section's premise, per
the bounded-outcome definition of "sent back"), this finding does not
survive as stated. The §4.5 sourcing gap on its own, isolated from the
broken §4.2 half, would be a fair "carry as an open question" item — but
that's not the finding as written.

---

## Summary

| # | Critic verdict | Refuter verdict | Basis |
|---|---|---|---|
| 1 | error | **SURVIVES** | grep confirms total silence; contract's named check is explicit and material |
| 2 | error | **SURVIVES** | grep confirms silence; proposal also omits ADR 0014's own named consequence (bootstrap hazard) |
| 3 | error | **REFUTED** | ADR 0014 — the cited fidelity authority — never records this correction; the finding's cited record doesn't say what it claims |
| 4 | warning | **SURVIVES (weak)** | outcome faithful, shape genuinely smoothed, matches critic's own mild calibration |
| 5 | warning | **SURVIVES (strong)** | genuine invented scope, hits the axis's core failure mode directly, no contract-checklist dependency |
| 6 | note | **REFUTED** | central claim (unattributed external quote) misreads the document's own blockquote convention; remainder is critic-conceded low-confidence |

Four of six findings survive refutation. Two do not: one (#3) because its
factual premise — that the fidelity authority records the correction it
demands — is false on inspection; one (#6) because its strongest half rests
on a misreading of the document's own formatting convention, and its
weaker half was already self-rated by the critic as probably not a real
break.

The strongest surviving finding is #5 (PACE/succession invented scope): it
is the one finding in this file that doesn't depend on the contract's
Captain-authored four-corrections checklist and instead hits the fidelity
axis's own stated failure mode directly, with no ADR 0014 backing anywhere
in the repository.
