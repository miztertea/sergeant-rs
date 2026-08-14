# FOUNDATION-1 — enactability refuter

Refuting `docs/gauntlet/runs/foundation-1/critics/enactability.md` against
`reference/proposal-foundation-rationalization.md`, ADRs 0005–0011, and the
repository as it stands. I did not write the proposal or the critic
findings. Method per `docs/gauntlet/contracts/FOUNDATION-1.md`: re-verify
each factual claim independently, check scope, check for style dressed as
defect, check whether the proposal already covers the gap elsewhere, check
severity.

---

## Finding 1 — §6 sequences §5.3 before §5.2; justification allegedly backwards

**Verdict: CONFIRMED, severity downgraded error → warning.**

**Re-verification.** I read §5.3(a), §6, §8.1, and ADR 0007 independently.
§5.3(a): "its environment guarantee (§5.2) and its execution model" —
attributes the guarantee to §5.2. ADR 0007(a): "alongside the environment
guarantee ADR 0006 establishes" — 0006 (=§5.2) establishes it, 0007 (=§5.3)
states it. §8.1: "§5.2 composes 'the environment,' and §5.3 states it." All
three agree: §5.2 establishes/composes, §5.3 states. This part of the
critic's evidence is accurate.

**Where the critic overstates it.** The critic's headline claim is that
§6's justification sentence itself "reads the dependency backwards." I
parsed that sentence on its own grammar: "§5.2 (passthrough) then, since
§5.3(a)'s environment guarantee is what the passthrough composes" reduces
to "the passthrough composes §5.3(a)'s environment guarantee" — i.e. §5.2
does the composing. That is not backwards; it is exactly consistent with
§8.1 and ADR 0007(a) (§5.2 establishes/composes). The critic's own quoted
evidence, read literally, does not support "reads the dependency
backwards" as a claim about the *sentence's logical content*. This part of
Finding 1 is refuted as argued.

**What survives.** The real defect is not the sentence's logic, it's the
*sequencing*: §6 places the "state" step (§5.3) before the "establish"
step (§5.2), so a Work executing step 2 would write actor-facing text
asserting a guarantee ("you have a deliberately composed environment")
that does not yet exist. That practical gap is real and I could not
resolve it away — the critic's two options (assert something false, or
silently ship only half of (a)) are the only ones the text supports.

**Does the proposal separate the two halves anywhere?** Not explicitly —
only §5.3(b) is marked "Independently" separable from (a); no equivalent
marker splits (a) itself. But §6's own two justifications *functionally*
perform this split without naming it: step 2's rationale for §5.3
("cheapest… stops active bleeding — actors have lost work twice") maps
1:1 onto #94, which §3.2 identifies as specifically the *execution-model*
failure ("Its execution model… Actors guessed otherwise twice"), not the
environment failure (#60, tied to §5.2). So the urgent, dispatchable part
of §5.3(a) that step 2 is actually justified by is the execution-model
half — already fully buildable with no dependency on §5.2. A competent
Work has textual grounds to infer the split even though it isn't spelled
out.

**Severity.** I checked this finding's structure against the critic's own
Finding 3, which is materially the same shape — a real contradiction in
one sub-clause of a section, while the section's other deliverables are
independently enactable — and Finding 3 was rated **warning**, not error,
specifically because only the narrow sub-question was blocked. Finding 1
fits the identical pattern: the (b) safety-net and the execution-model
half of (a) are fully dispatchable; only the environment-guarantee half of
(a) is blocked pending §5.2. Rating this "error" while rating the
structurally identical Finding 3 "warning" is an internal inconsistency in
the critic's own severity bar. I'm downgrading Finding 1 to **warning**,
carrying forward the critic's already-cheap fix (state which half of
§5.3(a) ships first, or swap step 2/3).

---

## Finding 2 — §5.2 presents environment composition as buildable while §8.1 admits it's unenumerated

**Verdict: CONFIRMED, severity downgraded error → warning; the "hides"
framing is refuted.**

**Re-verification.** §5.2: "sgt composes the environment, binds the
estate, and execs." §8.1, same document: "§5.2 composes 'the
environment'… The precise list — toolchain, estate binding, what else —
is not settled." ADR 0006's open questions: "The exact set of environment
variables and estate-binding facts the passthrough must compose was not
enumerated in the interview… this ADR records that the contract must
exist and be stated by the product, not its contents." All three match
the critic's quotes exactly. The factual claim is accurate.

**Where the critic overstates it: "hides... behind confident prose."**
The enactability axis's own test (per the contract) is whether a section
"hides an undecided question behind confident prose." §8.1 is a *named,
cross-referenced* Unknowns entry that opens by naming §5.2 specifically —
"§5.2 composes 'the environment'… is not settled." That is the opposite
of hiding: it is disclosure in the exact location the document's own
convention puts open questions, and §8's header says as much ("Named
rather than resolved, per the contract convention"). Compare the critic's
own Finding 4, which faults §5.1 for omitting a caveat that exists *only*
in ADR 0005 and has **no corresponding entry anywhere in the proposal's
own §8** — that is a real case of the proposal failing to carry a known
gap forward. Finding 2's gap, by contrast, *is* carried forward, verbatim,
into §8.1. The critic scored the fully-disclosed case (Finding 2) higher
severity (error) than the truly-undisclosed case (Finding 4, warning) —
backwards relative to the axis's own "hides… behind confident prose"
test. I'm refuting the "hides" characterization specifically.

**What survives independent of "hiding."** Disclosure isn't the whole
test, though — the axis also asks whether a section "has no acceptance
criterion a Work could satisfy." Naming an unknown in §8.1 doesn't supply
one. A Work dispatched against §5.2 alone still can't check whether it
composed "the environment" correctly, because no enumerable content
exists anywhere the document cites — that half of the finding holds
regardless of the hiding question, and the critic already scoped it
correctly ("the boundary is unaffected… but 'compose the environment' as
a deliverable is not dispatchable as stated"). I could not refute this
narrower claim: I checked whether ADR 0006's decision itself supplies a
minimal list (PATH? toolchain? just estate root?) and it does not —
"the contract must exist and be stated by the product, not its contents"
is explicit that content enumeration was deliberately deferred, not an
oversight the proposal introduced.

**Scope check.** Is this re-litigating ADR 0006 (deciding not to enumerate
content was the owner's call)? No — the contract explicitly puts "§8's
unknowns... in scope for this axis specifically," and the finding is about
whether §5.2's *text*, not the ruling itself, gives a Work something to
build against. That's in bounds.

**Severity.** Downgrading to **warning**: the estate-binding and exec
halves of §5.2 are fully dispatchable and reviewable (verified
independently — this matches the critic's own carve-out), only the
environment-variable-list sub-part is blocked, and that sub-part is
openly named rather than concealed. A Work here would have to invent or
defer something the proposal should have scoped more explicitly in §5.2's
own text (the axis's own definition of "warning"), not stall outright.

---

## Findings 3–6 — verified, not my primary assignment but checked

**Finding 3 (warning) — data_dir/surfaces_dir precedence contradiction.**
Re-verified directly: `src/cli.rs:391-399`'s doc comment confirms
`SGT_DATA_DIR` has unconditional precedence ahead of estate discovery;
`src/runtime/engine.rs:94-96` confirms `surfaces_root` prefers
`workspace.surfaces_dir` (manifest) over the engine default. The relative
orderings are opposite, as claimed. Checked §5.4 and all of §8 for a
carried-forward qualifier on this — none exists. **CONFIRMED as scored.**

**Finding 4 (warning) — §5.1 rebuild trigger has no operational bar.**
Re-verified: ADR 0005's open questions section states this gap verbatim
(`docs/adr/0005-gating-becomes-a-dispatched-work.md:105-110`); confirmed
`.sergeant/workflows/validate-and-ship/` exists, is `status: published`,
and has stages `40-drive-gates`/`50-reconcile-custody` matching §3.1's
citation. §8 has no entry for this gap. **CONFIRMED as scored** — and per
my own analysis above, this is genuinely the more severe pattern (gap
exists in the ADR, absent from the proposal's own §8), which is further
reason Finding 2's "error" rating (fully disclosed) sat oddly next to this
one's "warning" (undisclosed) before my reassessment.

**Finding 5 (warning) — §5.7 elides the m6 dashboard-test triage
ambiguity.** Re-verified directly:
`tests/m6_surfaces.rs:2460` (`t5_the_tui_and_the_dashboard_are_clients_
like_any_other`) loops `for module in ["tui.rs", "web.rs"]`; line 71
imports `DASHBOARD_CSS, DASHBOARD_JS` from `web.rs`; the function reads
`web.rs`'s source at line ~2491. Deleting `web.rs` breaks compilation of
this test, and the invariant it pins must survive for `tui.rs` alone —
confirming rewrite, not deletion, is needed for at least this one test.
§5.7 and §6 do not name this distinction; §8 has no entry for it.
**CONFIRMED as scored.**

**Finding 6 (info) — §5.4 "independent" overstates the precedence
rationale's dependency on §5.2.** Textual claim only; §5.4(a)'s own
sentence ("§5.2's explicit launch binding removes most of the surprise")
does condition the *rationale* on §5.2 while the coded deliverables
(field + re-ruling) don't. Already correctly scoped by the critic at
info — I found nothing to add or subtract. **CONFIRMED as scored.**

---

## Summary

| Finding | Critic severity | Refuter verdict | Refuter severity |
|---|---|---|---|
| 1 | error | CONFIRMED (framing partly refuted) | warning |
| 2 | error | CONFIRMED ("hides" framing refuted) | warning |
| 3 | warning | CONFIRMED | warning |
| 4 | warning | CONFIRMED | warning |
| 5 | warning | CONFIRMED | warning |
| 6 | info | CONFIRMED | info |

No finding in this axis is fully refuted on the facts — every factual
claim I re-checked against the repository held. Both error-severity
findings survive as real defects but neither survives at "error": both
follow the same shape as the critic's own warning-rated Finding 3 (a
narrow sub-clause is blocked, the section's main deliverables are not),
and the critic's own severity bar, applied consistently, puts them there
too. Finding 2 additionally mischaracterized disclosed-in-§8.1 as
"hidden," which I'm refuting as argued while the narrower "no acceptance
criterion" claim underneath it survives.
