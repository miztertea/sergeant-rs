# FOUNDATION-1 — fidelity refuter

Axis: fidelity. Assigned file: `docs/gauntlet/runs/foundation-1/critics/fidelity.md`.
All three findings land on §5.1 and ADR 0005. Re-read ADR 0005 in full
independently (not taking the critic's excerpts on trust), re-grepped the
codebase for the cited symbols, and re-read the whole proposal for
cross-references the critic's per-finding checks might have missed.

The "three corrections check" section confirms no finding for §5.3/§5.5/§5.7
— nothing to refute there, since the critic didn't confirm a finding.

## F1 — §5.1 states a gate-Work communication mechanism the ADR leaves undecided

**Verdict: CONFIRMED, but narrower than claimed — downgrade warning → info.**

Re-verified the factual predicate independently: `needs_input` is real
(`src/backend/mod.rs:570,599`, `src/daemon.rs:1011-1290`) and `Respond` is a
real CLI command (`src/cli.rs:115,575`). Re-read ADR 0005's Open Questions
section directly — the critic quotes it accurately: "what its own
findings-to-Work schema looks like... are not decided here."

Where the critic's read holds: the proposal states, unhedged, that ask-user
findings pause the gate Work via `needs_input` and resume via `sgt respond`.
That is one specific answer among plausible alternatives (e.g. batching
ask-user findings into the gate Work's final output for Captain to review
post-hoc, no mid-run hold at all) — and ADR 0005 explicitly declines to
settle "the mechanics of how a gate Work is specified," naming the
findings-to-Work schema as one of the undecided pieces. The critic is right
that this specific design choice is stated with the same confidence as
actual owner rulings, with no hedge.

Where the critic's read overreaches: they did not check `NORTH-STAR.md`,
which the proposal itself cites in §2.1 in the same breath as this exact
mechanism. R-NS-6 states: "sgt owns message *mechanics* to a running
execution (`needs_input`/`respond`, journaled)... never new hold machinery."
The proposal's phrase "the engine's existing hold mechanism, not new
machinery" is close to verbatim R-NS-6 language. That means the *transport*
half of F1's claim — which primitive is used, if a hold happens at all — is
not invented by the proposal; it traces to cited, already-ruled invariant
doctrine the same document references two sections earlier. The critic's
framing ("no cross-reference to §8's Unknowns... states a specific answer...
with no hedge") treats the whole sentence as ungrounded, when only half of
it is: *whether* an ask-user finding should interrupt the gate Work
mid-run at all (vs. surface only in completed output) is the genuinely open
part; *which* primitive would be used if it does is not.

**Correction to the critic's finding:** narrow the claim to "the proposal
decides that ask-user findings trigger a mid-run hold, which ADR 0005 does
not settle" rather than "the proposal invents an undecided communication
mechanism." The fix is even more local than the critic suggests — a
five-word hedge on the triggering question, not the whole sentence, and not
necessarily a move to §8 given the mechanism itself is already
NORTH-STAR-grounded.

## F2 — §5.1 drops the ADR's "negative consequence" framing

**Verdict: REFUTED — style preference dressed as a fidelity defect, plus a fabricated contract citation.**

First, a factual problem with the finding itself: it states "This is exactly
the pattern the contract names: 'An ADR that records a cost, summarized in
the proposal without it, is a fidelity defect.'" I grepped
`docs/gauntlet/contracts/FOUNDATION-1.md` for every substring of that quote
("records a cost", "summarized in the proposal", "fidelity defect") and none
of it appears anywhere in the contract. I also grepped the rest of `docs/`
and `reference/` — the phrase exists nowhere except in the critic's own
file. The critic invoked contract authority that doesn't exist. This is
exactly the failure mode a refuter exists to catch: a critic's own finding
inventing a citation to sound more grounded than it is.

Setting the fabricated citation aside and judging the substance: is this
"dropping a cost" or "summarizing"? Applying the distinction directly —
dropping a cost means the *fact* of the cost disappears; summarizing means
the fact survives with less editorial framing. Here the fact survives.
§5.1 states plainly that no-mistakes stays inside the gate Work, that a
rebuilt ICM review "is a new brief with no track record," and that "stages
get rebuilt only where we can show we have matched them." A reader gets the
same substantive information the ADR conveys — native ICM review isn't
replacing no-mistakes yet, dependency continues — without the ADR's
"negative consequence... does not go away" rhetorical label attached to it.
The critic's own write-up concedes this: "the reader can still infer the
dependency continues, but the proposal doesn't say so as plainly as the ADR
does." That sentence is describing a difference in plainness/emphasis, not
a missing fact — which is precisely the "I would have framed this
differently" pattern the refuter brief calls out as not a finding.

This doesn't mean §5.1's convention is perfectly consistent — it does label
costs explicitly elsewhere ("Consequence to watch"), and that labeled cost
covers a *different* risk (self-review independence, §8.2) than the ADR's
external-dependency cost. So there's a real minor inconsistency in which
costs get the explicit label. But inconsistent labeling of a fact that's
otherwise present is a style/completeness quibble, not a scope or
consequence error under this axis's definition ("say what was decided...
invented scope is the failure mode") — nothing is invented, and nothing
decided is actually missing from the reader's understanding.

## F3 — §5.1 omits ADR 0005's flagged gap in the "matched" criterion

**Verdict: CONFIRMED — survives as-is, no downgrade warranted.**

Re-verified independently: grepped the full proposal for "matched",
"evidence", "threshold", "side-by-side", "sign-off" — the only hit for
"matched" is the §5.1 sentence itself (line 237); the ADR's caveat about
what counts as evidence of a match is not restated anywhere, including §8's
Unknowns (re-read 8.1–8.5 in full: none address it).

This survives all five refutation grounds:
1. **Factually wrong?** No — ADR 0005's Open Questions section states this
   caveat verbatim, and the proposal's §5.1/§8 genuinely never restate it.
2. **Out of scope?** No — this is squarely fidelity-to-ADR-caveats, the
   axis's core remit, not implementation grading or re-litigating a ruling.
3. **Style preference?** No — this isn't "I'd have phrased it differently";
   it's a specific, checkable ADR sentence that's simply absent.
4. **Already said elsewhere?** No — checked all eight proposal sections;
   the only occurrence of "matched" in the whole document is the §5.1
   sentence the finding is about.
5. **Severity inflated?** No — the critic already self-rates this `info`,
   calls it "the weakest of the three findings here," and explicitly scopes
   it away from enactability. That's an honest, not inflated, severity call.

One point that actually *strengthens* F3 beyond what the critic wrote: §8
demonstrably carries forward other ADR open questions as a matter of the
proposal's own convention (§8.3 mirrors ADR 0010's homepage question, §8.4
mirrors ADR 0005's own "mechanics not decided" line) — the critic notes
this but undersells it. That §8 already has a §8.4 sourced from this *exact
ADR's* Open Questions section, one paragraph away from the "matched"
caveat, and picked up the neighboring sentence but not this one, makes the
omission look more like an oversight against the document's own stated
practice than a defensible editorial cut.

## Summary

| Finding | Verdict | Severity change |
|---|---|---|
| F1 | CONFIRMED, narrowed | warning → info |
| F2 | REFUTED | — |
| F3 | CONFIRMED | none (info, unchanged) |

F2's fabricated contract quote is worth flagging to adjudication
independent of the finding's merits: it's the kind of self-bolstering
citation that should be caught before it reaches an owner, regardless of
whether the underlying observation has value.
