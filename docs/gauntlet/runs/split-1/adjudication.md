# SPLIT-1 — adjudication

**Unit:** `docs/gauntlet/contracts/SPLIT-1.md`
**Artifact:** `reference/proposal-product-workspace-split.md` §1–§13
**Adjudicated:** 2026-08-17 by Captain (Opus)
**Seats:** 4 blind critics + 4 adversarial refuters, all `sonnet`, one turn
each, 1/45 turns spent per seat. ~30 minutes wall clock end to end.

## Verdict

**SENT BACK**, scoped to §2.

The contract's definition: *sent back — a confirmed finding invalidates a
section's premise, and the proposal is revised before enactment.* Two
subsections meet that bar:

- **§2.1** claims Six Memory Failure Modes scores "six for six" RED. The
  assumptions critic checked the routing-fog score's central evidence and
  found it contradicted; the refuter could not break that. §2.1's premise
  is the six-for-six claim, and the claim does not hold as written.
- **§2.2** claims 162 of 216 template files are contamination candidates.
  The figure has no stated methodology and does not reproduce; independent
  reconstruction lands ≈117.

**What is not sent back:** §3 (architecture), §4 (doctrine), §6 (phases),
§7 (gates), §13 (register) survive with fixable findings. The two-repository
topology, the three-substrate model, the co-versioning ruling, and the phase
dependencies are unchallenged in substance. This is a revision of the
evidentiary base, not of the plan.

**Adjudicator's conflict, declared.** Captain wrote the artifact, wrote the
contract, and is adjudicating. On the one genuinely close call — "validated
with findings" versus "sent back" — the less favourable reading was taken
deliberately, because the author is not positioned to grade generously.
Acceptance remains the owner's per the contract; this verdict is a
recommendation about the artifact's state, not a decision about the work.

## Tally

| Axis | Findings | Survived | Refuted |
|---|---|---|---|
| fidelity | 6 | 4 (one weakly) | 2 |
| invariants | 5 | 3 | 2 |
| assumptions | 9 | 9 | 0 |
| sequencing | 7 | 4 | 3 |
| **total** | **27** | **20** | **7** |

Two of the assumptions survivors are flagged by their own refuter as
non-adversarial — they confirm the proposal rather than find a defect. Net
defects carried: **18**.

## Surviving findings, by disposition

### Fix in the proposal before enactment

1. **§2.1's routing-fog score.** The claim that the bounded-judgment ladder
   "declares itself canonical in three places at once" is false. One
   self-declaration exists (`.sergeant/common/contexts/bounded-judgment.md`);
   `docs/icm/convention.md` explicitly *defers* to it — the fix for routing
   fog, cited as an instance of it. Rewrite the score's evidence or drop the
   score. The critic's characterisation — *"motivated reasoning toward a RED
   verdict the section had already committed to"* — is accepted.
2. **§2.1's dark-corner score** overclaims against `LESSONS.md` and
   `GAUNTLET.md`, which do provide partial homes for the context types the
   score calls homeless.
3. **§2.2's 162.** State a reproducible methodology or replace the headline
   with the two unambiguous counts (91 files referencing
   `reference/sergeant-upstream/`, 611 `BU-####` citations across 89 files),
   which hold exactly and independently carry the Phase 2 argument.
4. **§2.3's `@@ref` table.** Four counts are wrong: files citing
   `@@bounded-judgment` 77→79, occurrences 107→109, `@@tdd` 10→14, `@@name`
   6→8. Cause: inconsistent grep scope between `--include=CONTEXT.md` and
   `--include=*.md`. Also state that "80 stages" and "82 blocks" are drawn
   from different scopes.
5. **§1's LESSONS L20 citation** overstates what the record documents. The
   other two arrival-gap incidents check out exactly and stand.
6. **Fidelity's strongest survivor** — the four recorded corrections of
   Captain appear in ADR 0014 but nowhere in the proposal. Its refuter
   called this *"the strongest finding in the file."* Accepted: the
   proposal gains a short "Corrections to this proposal's own development"
   section. The anti-duplication rule is satisfied by summary-plus-pointer,
   not by silence.
7. **§4.4's PACE and succession** were invented without a ruling. **Mooted
   by owner ruling, 2026-08-17**, after the panel ran: both are adopted
   ("that military doctrine is sound"). The finding was correct when made;
   the defect is now the missing ADR trace, not the content. Record in ADR
   0014 as an amendment.
8. **Phase 3 carries no Ponytail rung**, which `docs/DEVELOPMENT.md`
   requires of every design decision. Add rung and justification.
9. **`GAUNTLET.md` and `LESSONS.md` are placed in neither repository nor
   substrate.** **Resolved by owner ruling, 2026-08-17:** both are
   decomposed into typed records in the workspace's OKF structure rather
   than moved intact — *"We don't just want to move the mess."* They become
   the first customer of the compile pass. Phase 4 grows accordingly.

### Fix in the phasing

10. **Phase 2 depends on a decision the gate table places after it.**
    Template decontamination needs the edition-marker format settled, which
    G3 places at Phase 3's entry. Reorder or split G3.
11. **Phase 5 bundles structural checks with the one check that needs
    Phase 4.** The structural validators can run against a single repo
    before the split; only the doctrine↔binary skew check needs both
    mounted. Split Phase 5.
12. **Three of seven gates are green-lit by the party performing the work
    behind them.** G2, G5, G6 list Captain as owner. At N=1 full separation
    is unavailable, but the proposal should state the degradation rather
    than imply independence — per its own §4.6 posture.
13. **PACE's Alternate branch** ("take the conservative option and record
    it") authorises a guess where the ladder requires a stop. Now that PACE
    is adopted doctrine, this needs rewriting so Alternate names a *route*,
    not a licence to decide.

### Refuted — no action

- Fidelity 3 (Gmail/inbox correction absent) and one other fidelity item.
- Invariants 3 (non-goals list) — *"immaterial, parasitic on Findings 1
  and 2."*
- **Invariants 4 (`sgt init` write path vs ADR 0008)** — *"mislabels an
  open-decision-list gap as an invariant violation, and the ADR 0008
  analogy is strained."* Captain reported this to the owner as a real catch
  before refutation; that report was premature and is corrected here.
- Sequencing 3 (§9's J2 grant) — refuted on the ground that a PR is a
  request, the label names the safeguard, and §9 separately lists merging
  as J0. The owner independently ruled the same way in the same hour:
  *"a pr is a request, which means it is not inherently approved…you may
  not accept that pull request into truth."* Converging independent
  arrivals; recorded as an ADR of its own per owner ruling.
- Sequencing 6 and 7 — immaterial. Finding 7's bootstrap observation (the
  overnight run lacks the ladder Phase 1 installs) is retained as a note,
  not a defect.

## Process observations

- **The empirical axis was the sharpest instrument.** Assumptions went 9/9
  against an adversary told to default to REFUTED. Every finding it made
  rested on a command the refuter could re-run. The axes resting on
  interpretation (fidelity, invariants) lost a third of their findings.
  Future proposal-grading units should weight empirical seats higher.
- **Two Captain reports to the owner were wrong before refutation ran** —
  the ADR 0008 catch (refuted) and the finding-3 authority problem
  (refuted, and separately overruled). Reporting unrefuted critic findings
  as conclusions is a defect in Captain's own conduct, not in the panel.
- **Dispatch defect:** all four refuter prompts contained a stray character
  ("it is真 but immaterial"). No refuter referenced it and all four produced
  coherent verdicts; recorded rather than concealed.
- **Third hand-rolled instance.** FOUNDATION-1, T-SERIES-1, and SPLIT-1 have
  now each reconstructed the same blind-critics → adversarial-refutation →
  adjudication shape from prose. Filed as a backlog row with a named
  trigger: **promote document-grading to a workflow package on the next
  proposal-grading unit.**

## Next

The proposal is revised against findings 1–13 before any phase begins.
Phase 0 is unaffected and proceeds by separate authority (ADR 0014; §6
scopes it independent of this verdict).
