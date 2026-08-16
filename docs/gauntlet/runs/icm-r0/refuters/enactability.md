# ICM-R0 — enactability refuter

Axis: enactability. Verifying `docs/gauntlet/runs/icm-r0/critics/enactability.md` (F1–F4)
against `reference/proposal-icm-r-procedure-authority.md` in full, independently — own
quotations pulled directly from the artifact, not from the critic's excerpts, and the
critic's chain of reasoning re-argued rather than assumed.

## Method

Read the proposal end to end (not the critic's summary of it) and re-derived each finding
from the cited sections. For F1, grepped every use of "adjudicat-" across the document to
test whether the proposal's own vocabulary resolves the ambiguity the critic alleges, rather
than trusting the critic's claim that it doesn't. For F2–F4, located the cited §19 decision,
the cited §10/§15/§9.5 text, and checked character-for-character whether a hedge or
cross-reference is actually absent, and separately asked whether the "no hedge" gap the
critic identifies is severe enough to threaten enactability rather than being a paperwork
nit. Where a finding could be attacked — argued into a lower severity or struck — I made that
argument in full before checking whether it survives.

## Findings

### F1 — verdict: CONFIRMED

**Critic's claim.** §10.1 lists "adjudicate the owner decisions in §19" as an Outcome bullet
for ICM-R0, but never says who performs that adjudication or how, creating a live conflict
with §19's own "does not silently make these owner rulings."

**Attempted refutation.** The strongest counter-argument is that "adjudicate" might carry a
narrower, self-consistent meaning elsewhere in the document — e.g., "surface as a
recommendation for review," which would dissolve the tension. I grepped every occurrence of
"adjudicat-" (28 hits) to test this. The results cut against the proposal, not for it:
line 44 states the whole ICM-R identifier is "subject to owner adjudication" (owner is the
adjudicator); line 1156 says the identifier "may [be renamed] during adjudication"; §8.12's
"Reconcile and publish" step says "The owner or delegated promotion gate accepts the final
disposition" — i.e., adjudication is consistently something an authority *external to the
producing process* performs on that process's output, not something the process does to
itself. §10.1's own sibling bullet, "adjudicate the two ladders and their precedence,"
appears in the same list and uses the identical verb for a different object (the ladders,
not §19), which does not resolve which sense governs the §19 bullet — if anything it shows
the document uses "adjudicate" loosely enough that a dispatched actor has no fixed referent
to resolve the ambiguity from internal vocabulary alone.

I also checked whether §19's own opening line is itself hedged enough to already cover this
("recommends defaults but does not silently make these owner rulings") — it is unconditional,
with no carve-out for the ICM-R0 workstream's own outcome bullet, so it directly contradicts
§10.1's literal claim rather than qualifying it.

Finally, I checked whether the tension is merely theoretical by confirming the contract had
to compensate for it externally: `docs/gauntlet/contracts/ICM-R0.md`'s Non-goals section
states "Not ruling on §19... This gauntlet does not make them either... never dispatched,
never decided by a panel" — language that only needs to exist because the proposal's own
§10.1 text, read at face value, assigns "adjudicate" to the workstream without qualification.
A document that already gets this right wouldn't need a separate contract to carve the
question back out.

**Verdict.** The critic's re-derivation holds under adversarial re-check. The ambiguity is
real, not resolved by the proposal's own internal vocabulary, and severity `error` is
justified: a dispatched Work with only §10.1's text (no external contract to patch the gap)
faces exactly the two-option dilemma the critic describes — silently settle the owner
rulings, or fail to complete "adjudicate" in any real sense.

### F2 — verdict: CONFIRMED

**Critic's claim.** §10.4's Subject list states "every shared context, helper, and
delegation they depend on" and "the built-in software-change workflow as a separate embedded
package" as unqualified in-scope subjects for ICM-R3, when §19 decision 3 lists exactly this
scope question ("Does 'every skill and workflow must be validated' include embedded
software-change, shared contexts, and helpers as first-class review subjects?") as an
unruled owner decision.

**Attempted refutation.** Two possible defenses: (a) §10.4's Subject list is describing what
the proposal *recommends* be in scope, consistent with decision 3's stated recommendation
("yes"), so stating it plainly isn't an error, only a style choice; (b) some other section of
§10 might carry the hedge the critic says is missing, making this a citation error. Checking
(a): §19 decision 3 explicitly separates "this proposal recommends X" from "the owner has
ruled X" — restating the recommendation as flat scope in the primary Acceptance-adjacent
Subject list erases that distinction for any reader/actor who only reads §10.4, which is
exactly what a dispatched Work would do to determine ICM-R3's boundaries. Checking (b): I
re-read all of §10.4 (Subject through Outcome) verbatim; no sentence in that subsection
references §19, "decision 3," "pending," or any conditional language. The only hedge
anywhere nearby is in §10.3's Outcome bullet ("No current package is moved merely because
§12 predicts its likely outcome"), which the critic already correctly excludes as a
different, adequately-hedged case in the "found nothing on" section — confirming the critic
did do this comparison rather than missing a hedge elsewhere in the document.

**Verdict.** Confirmed as written. The 23-workflow/4-skill portion of §10.4 is genuinely
unaffected (matches the critic's own scoping), but the embedded-workflow and
helper/shared-context lines state as settled exactly what §19-3 marks as owner-pending, with
no signal to a dispatched Work that this is contingent.

### F3 — verdict: CONFIRMED

**Critic's claim.** §9.5 states same-workflow-stage independence as flat operative doctrine
("A later stage may qualify when it receives only the artifact and review rubric...") while
§19 decision 7 asks the identical question and marks it "recommends yes," not ruled.

**Attempted refutation.** The candidate defense here is that §9.5's language is legitimately
definitional rather than a decision — i.e., it's just *defining what "independent" means*,
and decision 7 is a separate, narrower question about whether to *apply* that definition to
same-workflow stages. Re-reading §9.5's exact text refutes this: the sentence "A later stage
may qualify when it receives only the artifact and review rubric, does not inherit the
producing conversation, and cannot silently edit the subject it reviews" is precisely the
same-workflow-stage-qualifies claim, not a general definition independent of it — a
same-workflow stage is the paradigm case being described. And §19 decision 7's own text
("May a later stage in the same workflow qualify as independent when it has a fresh
execution, explicit inputs, a review-only contract, and no edit authority?") uses
near-identical criteria (fresh execution / explicit inputs / review-only / no edit authority)
to §9.5's (only artifact+rubric / no inherited conversation / cannot silently edit) — these
are restatements of the same test, not different questions. The defense doesn't survive
close reading.

I also checked whether §9.3 or §15 item 21, which the critic says rely on §9.5, actually
invoke it by name — they don't cite §9.5 explicitly, but §15 item 21 ("Every promotable
artifact names its independent review and promotion path") is unusable without some working
definition of "independent," and §9.5 is the only place in the document that supplies one.
The reliance is structural, not textual-citation, which is a fair reading, not an overreach.

**Verdict.** Confirmed as written. No hedge, no cross-reference to §19-7 anywhere in §9 or
§9.5.

### F4 — verdict: CONFIRMED

**Critic's claim.** The Executive Summary's hard boundary and §15 item 33 both state the
runtime freeze as flat, unconditional fact, while §19 decision 10 marks "hard contract or a
default that ... may interrupt" as owner-pending (with the proposal's own recommendation
being "hard contract").

**Attempted refutation.** The strongest defense: since the proposal's own recommended default
*is* "hard contract," and §15 items are elsewhere allowed to state recommended defaults as
operative (e.g., item 6's promotion-scope framing generally tracks recommended answers),
stating it flatly in item 33 is arguably harmless — a dispatched Work following item 33
literally can never go wrong, because "no src/ changes" is the conservative option regardless
of how decision 10 is eventually ruled. This is a real mitigant and is why the critic assigns
`warning`, not `error` — I checked and agree that's the right severity, not an overstatement.
But the defense doesn't fully dissolve the finding: the critic's actual point is narrower and
survives — decision 10 explicitly carves out one exception ("urgent runtime defects remain
separate work"), and neither the Executive Summary nor item 33 preserves that carve-out
language. A dispatched Work using §15 as a literal completion checklist (which is exactly
what §15's own framing instructs: "complete when all of the following are true") has no
textual signal that an urgent, independently-proven engine gap is anything other than an
absolute blocker through ICM-R4, contrary to what the proposal's own §19-10 text says it
intends to allow as a carve-out.

**Verdict.** Confirmed as written, at the `warning` severity the critic assigned — real gap,
correctly scoped (not inflated to `error`, since the practical failure mode is "an actor
can't act on an urgent defect it's not authorized to name," not "an actor stalls or guesses
wrong").

## Summary

All four findings survive independent re-derivation against the proposal's primary text.
None is struck; none requires a severity change from what the critic assigned. F1 is the
sharpest of the four — it is a genuine internal contradiction within the proposal's own
§10.1/§19 text, not merely an unhedged restatement of a recommended default (which is the
shared pattern in F2–F4).
