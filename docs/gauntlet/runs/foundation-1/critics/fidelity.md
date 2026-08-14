# FOUNDATION-1 — fidelity critic

Axis: does the proposal say what was decided, and only what was decided?
Every scope/rationale/consequence/commitment in §1–§8 was checked against
ADRs 0005–0011, the only source of the seven owner rulings, plus the
2026-08-11 arbitration/dispositions files where §3.6/§5.7 cite them.

## The three corrections check (contract's specific ask)

Read first, separately from the general sweep, because the contract calls
it out as the most serious failure mode: a proposal written by the
orchestrating session (Captain) laundering itself into having been right.

- **§5.3 / ADR 0007 (brief-provoked loss of work).** §3.2 states plainly:
  "The second occurrence was provoked by a dispatch brief written by
  Captain demanding empirical proof, which required long background runs.
  A well-formed acceptance criterion steered the actor into losing its
  work." This matches ADR 0007's context section, including the "brief
  author was not exempt from the problem it was diagnosing" framing.
  **Survives — not smoothed away.**
- **§5.5 / ADR 0009 (rejected TUI carve-out).** §5.5 states: "Captain
  proposed carving out the TUI as a human surface that should 'just work';
  that was rejected, and rightly — materializing a daemon so the cockpit
  has something to show is precisely the lie the rule exists to prevent."
  This matches ADR 0009's "the session agreed it was wrong on its own
  merits, not merely overruled." The proposal calls its own proposer wrong,
  not the owner merely stricter. **Survives.**
- **§5.7 / ADR 0011 (argument-record-was-not-a-ruling).** §3.6 and §5.7
  state the arbitration file "argued for deleting it… That file is the
  argument record, not the ruling," and "This is the first actual ruling
  on the dashboard, not a restatement." Checked against ADR 0011 and the
  2026-08-11 arbitration/dispositions files directly: the arbitration
  argument was one adversarial challenger's cut proposal
  (`north-star-arbitration-2026-08-11.md:190-196`, "CUT 3"), and the
  orchestrator's own dispositions record
  (`north-star-dispositions-2026-08-11.md`) never adopts it — "web,"
  "dashboard," and "freeze" do not appear in its four adopted items
  (D1–D4). The proposal does not claim more resolution existed than did.
  **Survives.**

All three hold. No finding here.

## Findings

### F1 — §5.1 states a gate-Work communication mechanism the ADR explicitly leaves undecided

- **severity:** warning
- **section:** §5.1 (Gating becomes a dispatched Work)
- **claim/text at issue:** "In a dispatched gate, `ask-user` surfaces as
  `needs_input` and Captain answers via `sgt respond` — the engine's
  existing hold mechanism, not new machinery."
- **what I checked:** ADR 0005's Decision and Open Questions sections;
  grepped the codebase for `needs_input` (`src/backend/mod.rs:570,599`,
  `src/daemon.rs`) and `Respond` (`src/cli.rs:115,575`) to confirm both
  exist today as generic engine machinery, not proposal-specific
  inventions.
- **what I found:** the mechanism named (`needs_input` signal, `sgt
  respond` command) is real and pre-existing, so this isn't a fabricated
  capability. But ADR 0005's own Open Questions section is explicit that
  this exact question is unruled: "The mechanics of how a gate Work is
  specified — what workflow stage invokes `scripts/gate.sh`/no-mistakes,
  **what its own findings-to-Work schema looks like**, whether it is a new
  named ICM workflow or a stage folded into an existing one — are not
  decided here; this ADR records that gating becomes a dispatched Work,
  not the shape of that Work." How an `ask-user` finding surfaces to
  Captain *is* the findings-to-Work schema question the ADR names as
  unresolved. The proposal states a specific answer to it with the same
  confidence as the parts of §5.1 that are actual owner rulings, with no
  hedge and no cross-reference to §8's Unknowns (where §8.4 discusses
  adjacent `validate-and-ship` re-homing questions but not this one).
- **does the section survive the correction:** yes. The core ruling in
  §5.1 (gate becomes a dispatched Work; Captain adjudicates, Sgt executes;
  the auto-fix/no-op/ask-user split is unchanged) is unaffected. The fix is
  local: hedge the `needs_input`/`sgt respond` sentence as one plausible
  shape given existing machinery, not a decided one, or move it to §8.

### F2 — §5.1 drops the ADR's explicit "negative consequence" framing for keeping no-mistakes embedded

- **severity:** warning
- **section:** §5.1 (Gating becomes a dispatched Work)
- **claim/text at issue:** "no-mistakes stays inside the gate Work
  initially. Its review is the asset… A rebuilt ICM review is a new brief
  with no track record. Stages get rebuilt only where we can show we have
  matched them."
- **what I checked:** ADR 0005's Consequences section against this
  paragraph and the rest of §5.1 for any equivalent framing.
- **what I found:** ADR 0005 records this same decision but explicitly as
  a cost, not only a rationale: "The negative consequence recorded
  alongside this decision is real: keeping no-mistakes embedded inside the
  gate Work… means this repo is still leaning on an external pipeline's
  ownership model for the one stage (review) that most needs a good cold
  reader. That dependency does not go away with this decision; it is
  deliberately kept…" The proposal keeps the positive rationale (track
  record, four real defects found) nearly verbatim from the ADR's
  Alternatives-considered section, but drops the ADR's own naming of this
  as a negative consequence that "does not go away." §5.1 does carry two
  other explicit costs ("Consequence accepted" / "Consequence to watch"
  for self-review), so the section's own convention for flagging costs
  exists — this one just isn't used here. This is exactly the pattern the
  contract names: "An ADR that records a cost, summarized in the proposal
  without it, is a fidelity defect."
- **does the section survive the correction:** yes. No scope changes;
  this is an omission of framing, not of substance — the reader can still
  infer the dependency continues, but the proposal doesn't say so as
  plainly as the ADR does. Restoring the ADR's "does not go away" framing
  alongside the existing "Consequence to watch" sentence would close this.

### F3 — §5.1 omits ADR 0005's flagged gap in its own "rebuild only where matched" criterion

- **severity:** info
- **section:** §5.1 (Gating becomes a dispatched Work); also absent from §8 (Unknowns)
- **claim/text at issue:** "Stages get rebuilt only where we can show we
  have matched them."
- **what I checked:** ADR 0005's Open Questions section against §5.1 and
  the full list of §8 Unknowns (8.1–8.5).
- **what I found:** ADR 0005 states this criterion and then immediately
  flags it as incomplete: "What specifically counts as evidence that a
  rebuilt stage has 'matched' the no-mistakes stage it would replace — a
  defect-count threshold, a side-by-side run, an owner sign-off — was not
  specified in the interview. Until that bar is named, 'rebuild only where
  we can show we have matched them' has no operational trigger and risks
  never firing, or firing on an ad hoc judgment call each time." The
  proposal states the criterion but not the ADR's own caveat about it, and
  §8's Unknowns list (which does carry forward other ADR open
  questions — e.g. §8.3 mirrors ADR 0010's homepage-estate-awareness open
  question, §8.4 mirrors ADR 0005's own mechanics-not-decided point)
  does not include this one.
- **does the section survive the correction:** yes. This is a minor
  completeness gap in an otherwise-faithful §8, not a scope or consequence
  error, and doesn't touch enactability of §5.1 itself (this axis's remit
  is fidelity to the ADRs, not whether the criterion is actionable — that
  is enactability's question). Flagged because the contract asks
  specifically whether stated ADR caveats survive; this one didn't, though
  it's the weakest of the three findings here.

## What I checked and found clean

Beyond the items above, I checked every §5.x subsection's stated scope,
rationale, and consequences against its cited ADR line by line (§5.2 vs
ADR 0006, §5.3 vs ADR 0007, §5.4 vs ADR 0008, §5.5 vs ADR 0009, §5.6 vs
ADR 0010, §5.7 vs ADR 0011), and the §1 mapping table's ADR references.
All six of those sections trace cleanly — no invented scope, no dropped
costs beyond F2/F3 above. Specifically checked and confirmed faithful:

- §5.4's "cost, stated rather than softened" language for re-ruling #64
  reproduces ADR 0008(c)'s own cost framing rather than dropping it.
- §5.5's "Blast radius, known" paragraph reproduces ADR 0009's three
  named consequences (AGENTS.md step 2, the three pinned tests by name,
  `sgt doctor`'s message going false) without omission.
- §5.6's quoted NORTH-STAR.md acceptance line ("a stranger reaches that
  last step in under five minutes of setup") matches `NORTH-STAR.md:25`
  verbatim — more precisely than ADR 0010's own paraphrase of the same
  line ("a finished change"), so no fidelity issue there.
- §5.6's "Open, not decided" line correctly preserves ADR 0010's own
  unruled status on homepage estate-awareness rather than presenting it
  as settled.
- The §1 table's seven §→ADR mappings are all correct.

No finding invalidates any section's premise. All three findings are
warning/info severity, none blocks §5.1 from being enacted as ruled — they
are about precision of a proposal that otherwise traces cleanly to its
ADRs.
